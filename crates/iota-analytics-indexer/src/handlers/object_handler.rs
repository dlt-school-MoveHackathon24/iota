// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use anyhow::Result;
use fastcrypto::encoding::{Base64, Encoding};
use iota_indexer::framework::Handler;
use iota_json_rpc_types::IotaMoveStruct;
use iota_package_resolver::Resolver;
use iota_rest_api::{CheckpointData, CheckpointTransaction};
use iota_types::{effects::TransactionEffects, object::Object};

use crate::{
    handlers::{
        get_move_struct, get_owner_address, get_owner_type, initial_shared_version,
        AnalyticsHandler, ObjectStatusTracker,
    },
    package_store::{LocalDBPackageStore, PackageCache},
    tables::{ObjectEntry, ObjectStatus},
    FileType,
};

pub struct ObjectHandler {
    objects: Vec<ObjectEntry>,
    package_store: LocalDBPackageStore,
    resolver: Resolver<PackageCache>,
}

#[async_trait::async_trait]
impl Handler for ObjectHandler {
    fn name(&self) -> &str {
        "object"
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
            self.process_transaction(
                checkpoint_summary.epoch,
                checkpoint_summary.sequence_number,
                checkpoint_summary.timestamp_ms,
                checkpoint_transaction,
                &checkpoint_transaction.effects,
            )
            .await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl AnalyticsHandler<ObjectEntry> for ObjectHandler {
    fn read(&mut self) -> Result<Vec<ObjectEntry>> {
        let cloned = self.objects.clone();
        self.objects.clear();
        Ok(cloned)
    }

    fn file_type(&self) -> Result<FileType> {
        Ok(FileType::Object)
    }
}

impl ObjectHandler {
    pub fn new(store_path: &Path, rest_uri: &str) -> Self {
        let package_store = LocalDBPackageStore::new(&store_path.join("object"), rest_uri);
        ObjectHandler {
            objects: vec![],
            package_store: package_store.clone(),
            resolver: Resolver::new(PackageCache::new(package_store)),
        }
    }
    async fn process_transaction(
        &mut self,
        epoch: u64,
        checkpoint: u64,
        timestamp_ms: u64,
        checkpoint_transaction: &CheckpointTransaction,
        effects: &TransactionEffects,
    ) -> Result<()> {
        let object_status_tracker = ObjectStatusTracker::new(effects);
        for object in checkpoint_transaction.output_objects.iter() {
            self.process_object(
                epoch,
                checkpoint,
                timestamp_ms,
                object,
                &object_status_tracker,
            )
            .await?;
        }
        for (object_ref, _) in effects.all_removed_objects().iter() {
            let entry = ObjectEntry {
                object_id: object_ref.0.to_string(),
                digest: object_ref.2.to_string(),
                version: u64::from(object_ref.1),
                type_: None,
                checkpoint,
                epoch,
                timestamp_ms,
                owner_type: None,
                owner_address: None,
                object_status: ObjectStatus::Deleted,
                initial_shared_version: None,
                previous_transaction: checkpoint_transaction.transaction.digest().base58_encode(),
                has_public_transfer: false,
                storage_rebate: None,
                bcs: None,
                coin_type: None,
                coin_balance: None,
                struct_tag: None,
                object_json: None,
            };
            self.objects.push(entry);
        }
        Ok(())
    }
    // Object data. Only called if there are objects in the transaction.
    // Responsible to build the live object table.
    async fn process_object(
        &mut self,
        epoch: u64,
        checkpoint: u64,
        timestamp_ms: u64,
        object: &Object,
        object_status_tracker: &ObjectStatusTracker,
    ) -> Result<()> {
        let move_obj_opt = object.data.try_as_move();
        let has_public_transfer = move_obj_opt
            .map(|o| o.has_public_transfer())
            .unwrap_or(false);
        let move_struct = if let Some((tag, contents)) = object
            .struct_tag()
            .and_then(|tag| object.data.try_as_move().map(|mo| (tag, mo.contents())))
        {
            let move_struct = get_move_struct(&tag, contents, &self.resolver).await?;
            Some(move_struct)
        } else {
            None
        };
        let (struct_tag, iota_move_struct) = if let Some(move_struct) = move_struct {
            match move_struct.into() {
                IotaMoveStruct::WithTypes { type_, fields } => {
                    (Some(type_), Some(IotaMoveStruct::WithFields(fields)))
                }
                fields => (object.struct_tag(), Some(fields)),
            }
        } else {
            (None, None)
        };
        let object_type = move_obj_opt.map(|o| o.type_().to_string());
        let object_id = object.id();
        let entry = ObjectEntry {
            object_id: object_id.to_string(),
            digest: object.digest().to_string(),
            version: object.version().value(),
            type_: object_type,
            checkpoint,
            epoch,
            timestamp_ms,
            owner_type: Some(get_owner_type(object)),
            owner_address: get_owner_address(object),
            object_status: object_status_tracker
                .get_object_status(&object_id)
                .expect("Object must be in output objects"),
            initial_shared_version: initial_shared_version(object),
            previous_transaction: object.previous_transaction.base58_encode(),
            has_public_transfer,
            storage_rebate: Some(object.storage_rebate),
            bcs: Some(Base64::encode(bcs::to_bytes(object).unwrap())),
            coin_type: object.coin_type_maybe().map(|t| t.to_string()),
            coin_balance: if object.coin_type_maybe().is_some() {
                Some(object.get_coin_value_unsafe())
            } else {
                None
            },
            struct_tag: struct_tag.map(|x| x.to_string()),
            object_json: iota_move_struct.map(|x| x.to_json_value().to_string()),
        };
        self.objects.push(entry);
        Ok(())
    }
}
