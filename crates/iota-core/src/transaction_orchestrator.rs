// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{path::Path, sync::Arc, time::Duration};

use futures::{
    future::{select, Either, Future},
    FutureExt,
};
use iota_common::sync::notify_read::NotifyRead;
use iota_metrics::{
    histogram::{Histogram, HistogramVec},
    spawn_logged_monitored_task, spawn_monitored_task, TX_TYPE_SHARED_OBJ_TX,
    TX_TYPE_SINGLE_WRITER_TX,
};
use iota_storage::write_path_pending_tx_log::WritePathPendingTransactionLog;
use iota_types::{
    base_types::TransactionDigest,
    effects::{TransactionEffectsAPI, VerifiedCertifiedTransactionEffects},
    error::{IotaError, IotaResult},
    executable_transaction::VerifiedExecutableTransaction,
    iota_system_state::IotaSystemState,
    quorum_driver_types::{
        ExecuteTransactionRequest, ExecuteTransactionRequestType, ExecuteTransactionResponse,
        FinalizedEffects, QuorumDriverEffectsQueueResult, QuorumDriverError, QuorumDriverResponse,
        QuorumDriverResult,
    },
    transaction::VerifiedTransaction,
};
use prometheus::{
    core::{AtomicI64, AtomicU64, GenericCounter, GenericGauge},
    register_int_counter_vec_with_registry, register_int_counter_with_registry,
    register_int_gauge_vec_with_registry, register_int_gauge_with_registry, Registry,
};
use tokio::{
    sync::broadcast::{error::RecvError, Receiver},
    task::JoinHandle,
    time::timeout,
};
use tracing::{debug, error, error_span, info, instrument, warn, Instrument};

use crate::{
    authority::{authority_per_epoch_store::AuthorityPerEpochStore, AuthorityState},
    authority_aggregator::{AuthAggMetrics, AuthorityAggregator},
    authority_client::{AuthorityAPI, NetworkAuthorityClient},
    quorum_driver::{
        reconfig_observer::{OnsiteReconfigObserver, ReconfigObserver},
        QuorumDriverHandler, QuorumDriverHandlerBuilder, QuorumDriverMetrics,
    },
    safe_client::SafeClientMetricsBase,
};

// How long to wait for local execution (including parents) before a timeout
// is returned to client.
const LOCAL_EXECUTION_TIMEOUT: Duration = Duration::from_secs(10);

const WAIT_FOR_FINALITY_TIMEOUT: Duration = Duration::from_secs(30);

/// Transaction Orchestrator is a Node component that utilizes Quorum Driver to
/// submit transactions to validators for finality, and proactively executes
/// finalized transactions locally, when possible.
pub struct TransactionOrchestrator<A: Clone> {
    quorum_driver_handler: Arc<QuorumDriverHandler<A>>,
    validator_state: Arc<AuthorityState>,
    _local_executor_handle: JoinHandle<()>,
    pending_tx_log: Arc<WritePathPendingTransactionLog>,
    notifier: Arc<NotifyRead<TransactionDigest, QuorumDriverResult>>,
    metrics: Arc<TransactionOrchestratorMetrics>,
}

impl TransactionOrchestrator<NetworkAuthorityClient> {
    pub fn new_with_network_clients(
        validator_state: Arc<AuthorityState>,
        reconfig_channel: Receiver<IotaSystemState>,
        parent_path: &Path,
        prometheus_registry: &Registry,
    ) -> anyhow::Result<Self> {
        let safe_client_metrics_base = SafeClientMetricsBase::new(prometheus_registry);
        let auth_agg_metrics = AuthAggMetrics::new(prometheus_registry);
        let validators = AuthorityAggregator::new_from_local_system_state(
            validator_state.get_cache_reader(),
            validator_state.committee_store(),
            safe_client_metrics_base.clone(),
            auth_agg_metrics.clone(),
        )?;

        let observer = OnsiteReconfigObserver::new(
            reconfig_channel,
            validator_state.get_cache_reader().clone(),
            validator_state.clone_committee_store(),
            safe_client_metrics_base,
            auth_agg_metrics,
        );
        Ok(TransactionOrchestrator::new(
            Arc::new(validators),
            validator_state,
            parent_path,
            prometheus_registry,
            observer,
        ))
    }
}

impl<A> TransactionOrchestrator<A>
where
    A: AuthorityAPI + Send + Sync + 'static + Clone,
    OnsiteReconfigObserver: ReconfigObserver<A>,
{
    pub fn new(
        validators: Arc<AuthorityAggregator<A>>,
        validator_state: Arc<AuthorityState>,
        parent_path: &Path,
        prometheus_registry: &Registry,
        reconfig_observer: OnsiteReconfigObserver,
    ) -> Self {
        let notifier = Arc::new(NotifyRead::new());
        let quorum_driver_handler = Arc::new(
            QuorumDriverHandlerBuilder::new(
                validators,
                Arc::new(QuorumDriverMetrics::new(prometheus_registry)),
            )
            .with_notifier(notifier.clone())
            .with_reconfig_observer(Arc::new(reconfig_observer))
            .start(),
        );

        let effects_receiver = quorum_driver_handler.subscribe_to_effects();
        let state_clone = validator_state.clone();
        let metrics = Arc::new(TransactionOrchestratorMetrics::new(prometheus_registry));
        let metrics_clone = metrics.clone();
        let pending_tx_log = Arc::new(WritePathPendingTransactionLog::new(
            parent_path.join("fullnode_pending_transactions"),
        ));
        let pending_tx_log_clone = pending_tx_log.clone();
        let _local_executor_handle = {
            spawn_monitored_task!(async move {
                Self::loop_execute_finalized_tx_locally(
                    state_clone,
                    effects_receiver,
                    pending_tx_log_clone,
                    metrics_clone,
                )
                .await;
            })
        };
        Self::schedule_txes_in_log(pending_tx_log.clone(), quorum_driver_handler.clone());
        Self {
            quorum_driver_handler,
            validator_state,
            _local_executor_handle,
            pending_tx_log,
            notifier,
            metrics,
        }
    }

    #[instrument(name = "tx_orchestrator_execute_transaction", level = "debug", skip_all,
    fields(
        tx_digest = ?request.transaction.digest(),
        tx_type = ?request.transaction_type(),
    ),
    err)]
    pub async fn execute_transaction_block(
        &self,
        request: ExecuteTransactionRequest,
    ) -> Result<ExecuteTransactionResponse, QuorumDriverError> {
        // TODO check if tx is already executed on this node.
        // Note: since EffectsCert is not stored today, we need to gather that from
        // validators (and maybe store it for caching purposes)
        let epoch_store = self.validator_state.load_epoch_store_one_call_per_task();

        let transaction = epoch_store
            .verify_transaction(request.transaction)
            .map_err(QuorumDriverError::InvalidUserSignature)?;
        let (_in_flight_metrics_guards, good_response_metrics) = self.update_metrics(&transaction);
        let tx_digest = *transaction.digest();
        debug!(?tx_digest, "TO Received transaction execution request.");

        let (_e2e_latency_timer, _txn_finality_timer) = if transaction.contains_shared_object() {
            (
                self.metrics.request_latency_shared_obj.start_timer(),
                self.metrics
                    .wait_for_finality_latency_shared_obj
                    .start_timer(),
            )
        } else {
            (
                self.metrics.request_latency_single_writer.start_timer(),
                self.metrics
                    .wait_for_finality_latency_single_writer
                    .start_timer(),
            )
        };

        // TODO: refactor all the gauge and timer metrics with `monitored_scope`
        let wait_for_finality_gauge = self.metrics.wait_for_finality_in_flight.clone();
        wait_for_finality_gauge.inc();
        let _wait_for_finality_gauge = scopeguard::guard(wait_for_finality_gauge, |in_flight| {
            in_flight.dec();
        });

        let ticket = self.submit(transaction.clone()).await.map_err(|e| {
            warn!(?tx_digest, "QuorumDriverInternalError: {e:?}");
            QuorumDriverError::QuorumDriverInternalError(e)
        })?;

        let wait_for_local_execution = matches!(
            request.request_type,
            ExecuteTransactionRequestType::WaitForLocalExecution
        );

        let Ok(result) = timeout(WAIT_FOR_FINALITY_TIMEOUT, ticket).await else {
            debug!(?tx_digest, "Timeout waiting for transaction finality.");
            self.metrics.wait_for_finality_timeout.inc();
            return Err(QuorumDriverError::TimeoutBeforeFinality);
        };

        drop(_txn_finality_timer);
        drop(_wait_for_finality_gauge);
        self.metrics.wait_for_finality_finished.inc();

        match result {
            Err(err) => {
                warn!(?tx_digest, "QuorumDriverInternalError: {err:?}");
                Err(QuorumDriverError::QuorumDriverInternalError(err))
            }
            Ok(Err(err)) => Err(err),
            Ok(Ok(response)) => {
                good_response_metrics.inc();
                let QuorumDriverResponse { effects_cert, .. } = response;
                if !wait_for_local_execution {
                    return Ok(ExecuteTransactionResponse::EffectsCert(Box::new((
                        FinalizedEffects::new_from_effects_cert(effects_cert.into()),
                        response.events,
                        false,
                    ))));
                }

                let executable_tx = VerifiedExecutableTransaction::new_from_quorum_execution(
                    transaction,
                    effects_cert.executed_epoch(),
                );

                match Self::execute_finalized_tx_locally_with_timeout(
                    &self.validator_state,
                    &epoch_store,
                    &executable_tx,
                    &effects_cert,
                    &self.metrics,
                )
                .await
                {
                    Ok(_) => Ok(ExecuteTransactionResponse::EffectsCert(Box::new((
                        FinalizedEffects::new_from_effects_cert(effects_cert.into()),
                        response.events,
                        true,
                    )))),
                    Err(_) => Ok(ExecuteTransactionResponse::EffectsCert(Box::new((
                        FinalizedEffects::new_from_effects_cert(effects_cert.into()),
                        response.events,
                        false,
                    )))),
                }
            }
        }
    }

    /// Submits the transaction to Quorum Driver for execution.
    /// Returns an awaitable Future.
    async fn submit(
        &self,
        transaction: VerifiedTransaction,
    ) -> IotaResult<impl Future<Output = IotaResult<QuorumDriverResult>> + '_> {
        let tx_digest = *transaction.digest();
        let ticket = self.notifier.register_one(&tx_digest);
        if self
            .pending_tx_log
            .write_pending_transaction_maybe(&transaction)
            .await?
        {
            debug!(?tx_digest, "no pending request in flight, submitting.");
            self.quorum_driver()
                .submit_transaction_no_ticket(transaction.clone().into())
                .await?;
        }
        // It's possible that the transaction effects is already stored in DB at this
        // point. So we also subscribe to that. If we hear from `effects_await`
        // first, it means the ticket misses the previous notification, and we
        // want to ask quorum driver to form a certificate for us again, to
        // serve this request.
        let cache_reader = self.validator_state.get_cache_reader().clone();
        let qd = self.clone_quorum_driver();
        Ok(async move {
            let digests = [tx_digest];
            let effects_await = cache_reader.notify_read_executed_effects(&digests);
            // let-and-return necessary to satisfy borrow checker.
            #[allow(clippy::let_and_return)]
            let res = match select(ticket, effects_await.boxed()).await {
                Either::Left((quorum_driver_response, _)) => Ok(quorum_driver_response),
                Either::Right((_, unfinished_quorum_driver_task)) => {
                    debug!(
                        ?tx_digest,
                        "Effects are available in DB, use quorum driver to get a certificate"
                    );
                    qd.submit_transaction_no_ticket(transaction.into()).await?;
                    Ok(unfinished_quorum_driver_task.await)
                }
            };
            res
        })
    }

    #[instrument(name = "tx_orchestrator_execute_finalized_tx_locally_with_timeout", level = "debug", skip_all, fields(tx_digest = ?transaction.digest()), err)]
    async fn execute_finalized_tx_locally_with_timeout(
        validator_state: &Arc<AuthorityState>,
        epoch_store: &Arc<AuthorityPerEpochStore>,
        transaction: &VerifiedExecutableTransaction,
        effects_cert: &VerifiedCertifiedTransactionEffects,
        metrics: &TransactionOrchestratorMetrics,
    ) -> IotaResult {
        // TODO: attempt a finalized tx at most once per request.
        // Every WaitForLocalExecution request will be attempted to execute twice,
        // one from the subscriber queue, one from the proactive execution before
        // returning results to clients. This is not insanely bad because:
        // 1. it's possible that one attempt finishes before the other, so there's zero
        //    extra work except DB checks
        // 2. an up-to-date fullnode should have minimal overhead to sync parents (for
        //    one extra time)
        // 3. at the end of day, the tx will be executed at most once per lock guard.
        let tx_digest = transaction.digest();
        if validator_state.is_tx_already_executed(tx_digest)? {
            return Ok(());
        }
        metrics.local_execution_in_flight.inc();
        let _metrics_guard =
            scopeguard::guard(metrics.local_execution_in_flight.clone(), |in_flight| {
                in_flight.dec();
            });

        let _guard = if transaction.contains_shared_object() {
            metrics.local_execution_latency_shared_obj.start_timer()
        } else {
            metrics.local_execution_latency_single_writer.start_timer()
        };
        debug!(?tx_digest, "Executing finalized tx locally.");
        match timeout(
            LOCAL_EXECUTION_TIMEOUT,
            validator_state.fullnode_execute_certificate_with_effects(
                transaction,
                effects_cert,
                epoch_store,
            ),
        )
        .instrument(error_span!(
            "transaction_orchestrator::local_execution",
            ?tx_digest
        ))
        .await
        {
            Err(_elapsed) => {
                debug!(
                    ?tx_digest,
                    "Executing tx locally by orchestrator timed out within {:?}.",
                    LOCAL_EXECUTION_TIMEOUT
                );
                metrics.local_execution_timeout.inc();
                Err(IotaError::Timeout)
            }
            Ok(Err(err)) => {
                debug!(
                    ?tx_digest,
                    "Executing tx locally by orchestrator failed with error: {:?}", err
                );
                metrics.local_execution_failure.inc();
                Err(IotaError::TransactionOrchestratorLocalExecution {
                    error: err.to_string(),
                })
            }
            Ok(Ok(_)) => {
                metrics.local_execution_success.inc();
                Ok(())
            }
        }
    }

    async fn loop_execute_finalized_tx_locally(
        validator_state: Arc<AuthorityState>,
        mut effects_receiver: Receiver<QuorumDriverEffectsQueueResult>,
        pending_transaction_log: Arc<WritePathPendingTransactionLog>,
        metrics: Arc<TransactionOrchestratorMetrics>,
    ) {
        loop {
            match effects_receiver.recv().await {
                Ok(Ok((transaction, QuorumDriverResponse { effects_cert, .. }))) => {
                    let tx_digest = transaction.digest();
                    if let Err(err) = pending_transaction_log.finish_transaction(tx_digest) {
                        panic!(
                            "Failed to finish transaction {tx_digest} in pending transaction log: {err}"
                        );
                    }

                    if transaction.contains_shared_object() {
                        // Do not locally execute transactions with shared objects, as this can
                        // cause forks until MVCC is merged.
                        continue;
                    }

                    let epoch_store = validator_state.load_epoch_store_one_call_per_task();

                    // This is a redundant verification, but SignatureVerifier will cache the
                    // previous result.
                    let transaction = match epoch_store.verify_transaction(transaction) {
                        Ok(transaction) => transaction,
                        Err(err) => {
                            // This should be impossible, since we verified the transaction
                            // before sending it to quorum driver.
                            error!(
                                ?err,
                                "Transaction signature failed to verify after quorum driver execution."
                            );
                            continue;
                        }
                    };

                    let executable_tx = VerifiedExecutableTransaction::new_from_quorum_execution(
                        transaction,
                        effects_cert.executed_epoch(),
                    );

                    let _ = Self::execute_finalized_tx_locally_with_timeout(
                        &validator_state,
                        &epoch_store,
                        &executable_tx,
                        &effects_cert,
                        &metrics,
                    )
                    .await;
                }
                Ok(Err((tx_digest, _err))) => {
                    if let Err(err) = pending_transaction_log.finish_transaction(&tx_digest) {
                        error!(
                            ?tx_digest,
                            "Failed to finish transaction in pending transaction log: {err}"
                        );
                    }
                }
                Err(RecvError::Closed) => {
                    error!("Sender of effects subscriber queue has been dropped!");
                    return;
                }
                Err(RecvError::Lagged(skipped_count)) => {
                    warn!("Skipped {skipped_count} transasctions in effects subscriber queue.");
                }
            }
        }
    }

    pub fn quorum_driver(&self) -> &Arc<QuorumDriverHandler<A>> {
        &self.quorum_driver_handler
    }

    pub fn clone_quorum_driver(&self) -> Arc<QuorumDriverHandler<A>> {
        self.quorum_driver_handler.clone()
    }

    pub fn clone_authority_aggregator(&self) -> Arc<AuthorityAggregator<A>> {
        self.quorum_driver().authority_aggregator().load_full()
    }

    pub fn subscribe_to_effects_queue(&self) -> Receiver<QuorumDriverEffectsQueueResult> {
        self.quorum_driver_handler.subscribe_to_effects()
    }

    fn update_metrics(
        &'_ self,
        transaction: &VerifiedTransaction,
    ) -> (impl Drop, &'_ GenericCounter<AtomicU64>) {
        let (in_flight, good_response) = if transaction.contains_shared_object() {
            self.metrics.total_req_received_shared_object.inc();
            (
                self.metrics.req_in_flight_shared_object.clone(),
                &self.metrics.good_response_shared_object,
            )
        } else {
            self.metrics.total_req_received_single_writer.inc();
            (
                self.metrics.req_in_flight_single_writer.clone(),
                &self.metrics.good_response_single_writer,
            )
        };
        in_flight.inc();
        (
            scopeguard::guard(in_flight, |in_flight| {
                in_flight.dec();
            }),
            good_response,
        )
    }

    fn schedule_txes_in_log(
        pending_tx_log: Arc<WritePathPendingTransactionLog>,
        quorum_driver: Arc<QuorumDriverHandler<A>>,
    ) {
        spawn_logged_monitored_task!(async move {
            if std::env::var("SKIP_LOADING_FROM_PENDING_TX_LOG").is_ok() {
                info!("Skipping loading pending transactions from pending_tx_log.");
                return;
            }
            let pending_txes = pending_tx_log.load_all_pending_transactions();
            info!(
                "Recovering {} pending transactions from pending_tx_log.",
                pending_txes.len()
            );
            for (i, tx) in pending_txes.into_iter().enumerate() {
                // TODO: ideally pending_tx_log would not contain VerifiedTransaction, but that
                // requires a migration.
                let tx = tx.into_inner();
                let tx_digest = *tx.digest();
                // It's not impossible we fail to enqueue a task but that's not the end of
                // world.
                if let Err(err) = quorum_driver.submit_transaction_no_ticket(tx).await {
                    warn!(
                        ?tx_digest,
                        "Failed to enqueue transaction from pending_tx_log, err: {err:?}"
                    );
                } else {
                    debug!(?tx_digest, "Enqueued transaction from pending_tx_log");
                    if (i + 1) % 1000 == 0 {
                        info!("Enqueued {} transactions from pending_tx_log.", i + 1);
                    }
                }
            }
            // Transactions will be cleaned up in
            // loop_execute_finalized_tx_locally() after they
            // produce effects.
        });
    }

    pub fn load_all_pending_transactions(&self) -> Vec<VerifiedTransaction> {
        self.pending_tx_log.load_all_pending_transactions()
    }
}

/// Prometheus metrics which can be displayed in Grafana, queried and alerted on
#[derive(Clone)]
pub struct TransactionOrchestratorMetrics {
    total_req_received_single_writer: GenericCounter<AtomicU64>,
    total_req_received_shared_object: GenericCounter<AtomicU64>,

    good_response_single_writer: GenericCounter<AtomicU64>,
    good_response_shared_object: GenericCounter<AtomicU64>,

    req_in_flight_single_writer: GenericGauge<AtomicI64>,
    req_in_flight_shared_object: GenericGauge<AtomicI64>,

    wait_for_finality_in_flight: GenericGauge<AtomicI64>,
    wait_for_finality_finished: GenericCounter<AtomicU64>,
    wait_for_finality_timeout: GenericCounter<AtomicU64>,

    local_execution_in_flight: GenericGauge<AtomicI64>,
    local_execution_success: GenericCounter<AtomicU64>,
    local_execution_timeout: GenericCounter<AtomicU64>,
    local_execution_failure: GenericCounter<AtomicU64>,

    request_latency_single_writer: Histogram,
    request_latency_shared_obj: Histogram,
    wait_for_finality_latency_single_writer: Histogram,
    wait_for_finality_latency_shared_obj: Histogram,
    local_execution_latency_single_writer: Histogram,
    local_execution_latency_shared_obj: Histogram,
}

// Note that labeled-metrics are stored upfront individually
// to mitigate the perf hit by MetricsVec.
// See https://github.com/tikv/rust-prometheus/tree/master/static-metric
impl TransactionOrchestratorMetrics {
    pub fn new(registry: &Registry) -> Self {
        let total_req_received = register_int_counter_vec_with_registry!(
            "tx_orchestrator_total_req_received",
            "Total number of executions request Transaction Orchestrator receives, group by tx type",
            &["tx_type"],
            registry
        )
        .unwrap();

        let total_req_received_single_writer =
            total_req_received.with_label_values(&[TX_TYPE_SINGLE_WRITER_TX]);
        let total_req_received_shared_object =
            total_req_received.with_label_values(&[TX_TYPE_SHARED_OBJ_TX]);

        let good_response = register_int_counter_vec_with_registry!(
            "tx_orchestrator_good_response",
            "Total number of good responses Transaction Orchestrator generates, group by tx type",
            &["tx_type"],
            registry
        )
        .unwrap();

        let good_response_single_writer =
            good_response.with_label_values(&[TX_TYPE_SINGLE_WRITER_TX]);
        let good_response_shared_object = good_response.with_label_values(&[TX_TYPE_SHARED_OBJ_TX]);

        let req_in_flight = register_int_gauge_vec_with_registry!(
            "tx_orchestrator_req_in_flight",
            "Number of requests in flights Transaction Orchestrator processes, group by tx type",
            &["tx_type"],
            registry
        )
        .unwrap();

        let req_in_flight_single_writer =
            req_in_flight.with_label_values(&[TX_TYPE_SINGLE_WRITER_TX]);
        let req_in_flight_shared_object = req_in_flight.with_label_values(&[TX_TYPE_SHARED_OBJ_TX]);

        let request_latency = HistogramVec::new_in_registry(
            "tx_orchestrator_request_latency",
            "Time spent in processing one Transaction Orchestrator request",
            &["tx_type"],
            registry,
        );
        let wait_for_finality_latency = HistogramVec::new_in_registry(
            "tx_orchestrator_wait_for_finality_latency",
            "Time spent in waiting for one Transaction Orchestrator request gets finalized",
            &["tx_type"],
            registry,
        );
        let local_execution_latency = HistogramVec::new_in_registry(
            "tx_orchestrator_local_execution_latency",
            "Time spent in waiting for one Transaction Orchestrator gets locally executed",
            &["tx_type"],
            registry,
        );

        Self {
            total_req_received_single_writer,
            total_req_received_shared_object,
            good_response_single_writer,
            good_response_shared_object,
            req_in_flight_single_writer,
            req_in_flight_shared_object,
            wait_for_finality_in_flight: register_int_gauge_with_registry!(
                "tx_orchestrator_wait_for_finality_in_flight",
                "Number of in flight txns Transaction Orchestrator are waiting for finality for",
                registry,
            )
            .unwrap(),
            wait_for_finality_finished: register_int_counter_with_registry!(
                "tx_orchestrator_wait_for_finality_fnished",
                "Total number of txns Transaction Orchestrator gets responses from Quorum Driver before timeout, either success or failure",
                registry,
            )
            .unwrap(),
            wait_for_finality_timeout: register_int_counter_with_registry!(
                "tx_orchestrator_wait_for_finality_timeout",
                "Total number of txns timing out in waiting for finality Transaction Orchestrator handles",
                registry,
            )
            .unwrap(),
            local_execution_in_flight: register_int_gauge_with_registry!(
                "tx_orchestrator_local_execution_in_flight",
                "Number of local execution txns in flights Transaction Orchestrator handles",
                registry,
            )
            .unwrap(),
            local_execution_success: register_int_counter_with_registry!(
                "tx_orchestrator_local_execution_success",
                "Total number of successful local execution txns Transaction Orchestrator handles",
                registry,
            )
            .unwrap(),
            local_execution_timeout: register_int_counter_with_registry!(
                "tx_orchestrator_local_execution_timeout",
                "Total number of timed-out local execution txns Transaction Orchestrator handles",
                registry,
            )
            .unwrap(),
            local_execution_failure: register_int_counter_with_registry!(
                "tx_orchestrator_local_execution_failure",
                "Total number of failed local execution txns Transaction Orchestrator handles",
                registry,
            )
            .unwrap(),
            request_latency_single_writer: request_latency
                .with_label_values(&[TX_TYPE_SINGLE_WRITER_TX]),
            request_latency_shared_obj: request_latency.with_label_values(&[TX_TYPE_SHARED_OBJ_TX]),
            wait_for_finality_latency_single_writer: wait_for_finality_latency
                .with_label_values(&[TX_TYPE_SINGLE_WRITER_TX]),
            wait_for_finality_latency_shared_obj: wait_for_finality_latency
                .with_label_values(&[TX_TYPE_SHARED_OBJ_TX]),
            local_execution_latency_single_writer: local_execution_latency
                .with_label_values(&[TX_TYPE_SINGLE_WRITER_TX]),
            local_execution_latency_shared_obj: local_execution_latency
                .with_label_values(&[TX_TYPE_SHARED_OBJ_TX]),
        }
    }

    pub fn new_for_tests() -> Self {
        let registry = Registry::new();
        Self::new(&registry)
    }
}
