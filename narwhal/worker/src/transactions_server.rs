// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use iota_metrics::{metered_channel::Sender, monitored_scope, spawn_logged_monitored_task};
use iota_network_stack::{server::Server, Multiaddr};
use tokio::{
    task::JoinHandle,
    time::{sleep, timeout},
};
use tonic::{Request, Response, Status};
use tracing::{error, info, warn};
use types::{
    ConditionalBroadcastReceiver, Empty, Transaction, TransactionProto, Transactions,
    TransactionsServer, TxResponse,
};

use crate::{client::LocalNarwhalClient, metrics::WorkerEndpointMetrics, TransactionValidator};

pub struct TxServer<V: TransactionValidator> {
    address: Multiaddr,
    rx_shutdown: ConditionalBroadcastReceiver,
    endpoint_metrics: WorkerEndpointMetrics,
    local_client: Arc<LocalNarwhalClient>,
    validator: V,
}

impl<V: TransactionValidator> TxServer<V> {
    #[must_use]
    pub fn spawn(
        address: Multiaddr,
        rx_shutdown: ConditionalBroadcastReceiver,
        endpoint_metrics: WorkerEndpointMetrics,
        tx_batch_maker: Sender<(Transaction, TxResponse)>,
        validator: V,
    ) -> JoinHandle<()> {
        // create and initialize local Narwhal client.
        let local_client = LocalNarwhalClient::new(tx_batch_maker);
        LocalNarwhalClient::set_global(address.clone(), local_client.clone());

        spawn_logged_monitored_task!(
            Self {
                address,
                rx_shutdown,
                endpoint_metrics,
                local_client,
                validator,
            }
            .run(),
            "TxServer"
        )
    }

    async fn run(mut self) {
        const MAX_RETRIES: usize = 10;
        const RETRY_BACKOFF: Duration = Duration::from_millis(1_000);
        const GRACEFUL_SHUTDOWN_DURATION: Duration = Duration::from_millis(2_000);

        // create the handler
        let tx_handler = TxReceiverHandler {
            local_client: self.local_client.clone(),
            validator: self.validator,
        };

        // now create the server
        let mut retries = MAX_RETRIES;
        let mut server: Server;

        loop {
            match iota_network_stack::config::Config::new()
                .server_builder_with_metrics(self.endpoint_metrics.clone())
                .add_service(TransactionsServer::new(tx_handler.clone()))
                .bind(&self.address)
                .await
            {
                Ok(s) => {
                    server = s;
                    break;
                }
                Err(err) => {
                    retries -= 1;
                    if retries == 0 {
                        panic!(
                            "Couldn't boot transactions server, permanently failed: {}",
                            err
                        );
                    }

                    error!(
                        "Couldn't boot transactions server at try {}, will wait {}s and retry: {}",
                        retries,
                        RETRY_BACKOFF.as_secs_f64(),
                        err
                    );

                    sleep(RETRY_BACKOFF).await;
                }
            }
        }

        let shutdown_handle = server.take_cancel_handle().unwrap();

        let server_handle = spawn_logged_monitored_task!(server.serve());

        // wait to receive a shutdown signal
        let _ = self.rx_shutdown.receiver.recv().await;

        // once do just gracefully signal the node to shutdown
        shutdown_handle.send(()).unwrap();

        // now wait until the handle completes or timeout if it takes long time
        match timeout(GRACEFUL_SHUTDOWN_DURATION, server_handle).await {
            Ok(_) => {
                info!("Successfully shutting down gracefully transactions server");
            }
            Err(err) => {
                warn!(
                    "Time out while waiting to gracefully shutdown transactions server: {}",
                    err
                )
            }
        }
    }
}

/// Defines how the network receiver handles incoming transactions.
#[derive(Clone)]
pub(crate) struct TxReceiverHandler<V> {
    pub(crate) local_client: Arc<LocalNarwhalClient>,
    pub(crate) validator: V,
}

#[async_trait]
impl<V: TransactionValidator> Transactions for TxReceiverHandler<V> {
    async fn submit_transaction(
        &self,
        request: Request<TransactionProto>,
    ) -> Result<Response<Empty>, Status> {
        let _scope = monitored_scope("SubmitTransaction");
        let transaction = request.into_inner().transaction;

        let validate_scope = monitored_scope("SubmitTransaction_ValidateTx");
        if self.validator.validate(transaction.as_ref()).is_err() {
            return Err(Status::invalid_argument("Invalid transaction"));
        }
        drop(validate_scope);

        // Send the transaction to Narwhal via the local client.
        let submit_scope = monitored_scope("SubmitTransaction_SubmitTx");
        self.local_client
            .submit_transaction(transaction.to_vec())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        drop(submit_scope);
        Ok(Response::new(Empty {}))
    }

    async fn submit_transaction_stream(
        &self,
        request: Request<tonic::Streaming<types::TransactionProto>>,
    ) -> Result<Response<types::Empty>, Status> {
        let mut transactions = request.into_inner();
        let mut requests = FuturesUnordered::new();

        let _scope = monitored_scope("SubmitTransactionStream");
        while let Some(Ok(txn)) = transactions.next().await {
            let validate_scope = monitored_scope("SubmitTransactionStream_ValidateTx");
            if let Err(err) = self.validator.validate(txn.transaction.as_ref()) {
                // If the transaction is invalid (often cryptographically), better to drop the
                // client
                return Err(Status::invalid_argument(format!(
                    "Stream contains an invalid transaction {err}"
                )));
            }
            drop(validate_scope);
            // Send the transaction to Narwhal via the local client.
            // Note that here we do not wait for a response because this would
            // mean that we process only a single message from this stream at a
            // time. Instead we gather them and resolve them once the stream is over.
            let submit_scope = monitored_scope("SubmitTransactionStream_SubmitTx");
            requests.push(
                self.local_client
                    .submit_transaction(txn.transaction.to_vec()),
            );
            drop(submit_scope);
        }

        while let Some(result) = requests.next().await {
            if let Err(e) = result {
                return Err(Status::internal(e.to_string()));
            }
        }

        Ok(Response::new(Empty {}))
    }
}
