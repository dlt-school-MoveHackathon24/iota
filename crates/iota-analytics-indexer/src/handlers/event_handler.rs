// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use anyhow::Result;
use fastcrypto::encoding::{Base64, Encoding};
use iota_indexer::framework::Handler;
use iota_json_rpc_types::IotaMoveStruct;
use iota_package_resolver::Resolver;
use iota_rest_api::CheckpointData;
use iota_types::{digests::TransactionDigest, effects::TransactionEvents, event::Event};

use crate::{
    handlers::{get_move_struct, AnalyticsHandler},
    package_store::{LocalDBPackageStore, PackageCache},
    tables::EventEntry,
    FileType,
};

pub struct EventHandler {
    events: Vec<EventEntry>,
    package_store: LocalDBPackageStore,
    resolver: Resolver<PackageCache>,
}

#[async_trait::async_trait]
impl Handler for EventHandler {
    fn name(&self) -> &str {
        "event"
    }
    async fn process_checkpoint(&mut self, checkpoint_data: &CheckpointData) -> Result<()> {
        let CheckpointData {
            checkpoint_summary,
            transactions: checkpoint_transactions,
            ..
        } = checkpoint_data;
        for checkpoint_transaction in checkpoint_transactions {
            for object in checkpoint_transaction.output_objects.iter() {
                self.package_store.update(object)?;
            }
            if let Some(events) = &checkpoint_transaction.events {
                self.process_events(
                    checkpoint_summary.epoch,
                    checkpoint_summary.sequence_number,
                    checkpoint_transaction.transaction.digest(),
                    checkpoint_summary.timestamp_ms,
                    events,
                )
                .await?;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl AnalyticsHandler<EventEntry> for EventHandler {
    fn read(&mut self) -> Result<Vec<EventEntry>> {
        let cloned = self.events.clone();
        self.events.clear();
        Ok(cloned)
    }

    fn file_type(&self) -> Result<FileType> {
        Ok(FileType::Event)
    }
}

impl EventHandler {
    pub fn new(store_path: &Path, rest_uri: &str) -> Self {
        let package_store = LocalDBPackageStore::new(&store_path.join("event"), rest_uri);
        EventHandler {
            events: vec![],
            package_store: package_store.clone(),
            resolver: Resolver::new(PackageCache::new(package_store)),
        }
    }
    async fn process_events(
        &mut self,
        epoch: u64,
        checkpoint: u64,
        digest: &TransactionDigest,
        timestamp_ms: u64,
        events: &TransactionEvents,
    ) -> Result<()> {
        for (idx, event) in events.data.iter().enumerate() {
            let Event {
                package_id,
                transaction_module,
                sender,
                type_,
                contents,
            } = event;
            let move_struct = get_move_struct(type_, contents, &self.resolver).await?;
            let (_struct_tag, iota_move_struct) = match move_struct.into() {
                IotaMoveStruct::WithTypes { type_, fields } => {
                    (type_, IotaMoveStruct::WithFields(fields))
                }
                fields => (type_.clone(), fields),
            };
            let event_json = iota_move_struct.to_json_value();
            let entry = EventEntry {
                transaction_digest: digest.base58_encode(),
                event_index: idx as u64,
                checkpoint,
                epoch,
                timestamp_ms,
                sender: sender.to_string(),
                package: package_id.to_string(),
                module: transaction_module.to_string(),
                event_type: type_.to_string(),
                bcs: Base64::encode(contents.clone()),
                event_json: event_json.to_string(),
            };

            self.events.push(entry);
        }
        Ok(())
    }
}
