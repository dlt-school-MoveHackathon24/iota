// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, path::Path};

use anyhow::Result;
use fastcrypto::encoding::{Base64, Encoding};
use iota_indexer::{errors::IndexerError, framework::Handler, types::owner_to_owner_info};
use iota_json_rpc_types::IotaMoveValue;
use iota_package_resolver::Resolver;
use iota_rest_api::{CheckpointData, CheckpointTransaction};
use iota_types::{
    base_types::ObjectID,
    dynamic_field::{DynamicFieldInfo, DynamicFieldName, DynamicFieldType},
    object::Object,
};
use tap::tap::TapFallible;
use tracing::warn;

use crate::{
    handlers::{get_move_struct, AnalyticsHandler},
    package_store::{LocalDBPackageStore, PackageCache},
    tables::DynamicFieldEntry,
    FileType,
};

pub struct DynamicFieldHandler {
    dynamic_fields: Vec<DynamicFieldEntry>,
    package_store: LocalDBPackageStore,
    resolver: Resolver<PackageCache>,
}

#[async_trait::async_trait]
impl Handler for DynamicFieldHandler {
    fn name(&self) -> &str {
        "dynamic_field"
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
            )
            .await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl AnalyticsHandler<DynamicFieldEntry> for DynamicFieldHandler {
    fn read(&mut self) -> Result<Vec<DynamicFieldEntry>> {
        let cloned = self.dynamic_fields.clone();
        self.dynamic_fields.clear();
        Ok(cloned)
    }

    fn file_type(&self) -> Result<FileType> {
        Ok(FileType::DynamicField)
    }
}

impl DynamicFieldHandler {
    pub fn new(store_path: &Path, rest_uri: &str) -> Self {
        let package_store = LocalDBPackageStore::new(&store_path.join("dynamic_field"), rest_uri);
        DynamicFieldHandler {
            dynamic_fields: vec![],
            package_store: package_store.clone(),
            resolver: Resolver::new(PackageCache::new(package_store)),
        }
    }
    async fn process_dynamic_field(
        &mut self,
        epoch: u64,
        checkpoint: u64,
        timestamp_ms: u64,
        object: &Object,
        all_written_objects: &HashMap<ObjectID, Object>,
    ) -> Result<()> {
        let move_obj_opt = object.data.try_as_move();
        // Skip if not a move object
        let Some(move_object) = move_obj_opt else {
            return Ok(());
        };
        if !move_object.type_().is_dynamic_field() {
            return Ok(());
        }
        let move_struct = if let Some((tag, contents)) = object
            .struct_tag()
            .and_then(|tag| object.data.try_as_move().map(|mo| (tag, mo.contents())))
        {
            let move_struct = get_move_struct(&tag, contents, &self.resolver).await?;
            Some(move_struct)
        } else {
            None
        };
        let Some(move_struct) = move_struct else {
            return Ok(());
        };
        let (name_value, type_, object_id) =
            DynamicFieldInfo::parse_move_object(&move_struct).tap_err(|e| warn!("{e}"))?;
        let name_type = move_object.type_().try_extract_field_name(&type_)?;

        let bcs_name = bcs::to_bytes(&name_value.clone().undecorate()).map_err(|e| {
            IndexerError::SerdeError(format!(
                "Failed to serialize dynamic field name {:?}: {e}",
                name_value
            ))
        })?;
        let name = DynamicFieldName {
            type_: name_type,
            value: IotaMoveValue::from(name_value).to_json_value(),
        };
        let name_json = serde_json::to_string(&name)?;
        let (_owner_type, owner_id) = owner_to_owner_info(&object.owner);
        let Some(parent_id) = owner_id else {
            return Ok(());
        };
        let entry = match type_ {
            DynamicFieldType::DynamicField => DynamicFieldEntry {
                parent_object_id: parent_id.to_string(),
                transaction_digest: object.previous_transaction.base58_encode(),
                checkpoint,
                epoch,
                timestamp_ms,
                name: name_json,
                bcs_name: Base64::encode(bcs_name),
                type_,
                object_id: object.id().to_string(),
                version: object.version().value(),
                digest: object.digest().to_string(),
                object_type: move_object.clone().into_type().into_type_params()[1]
                    .to_canonical_string(/* with_prefix */ true),
            },
            DynamicFieldType::DynamicObject => {
                let object = all_written_objects.get(&object_id).ok_or(
                    IndexerError::UncategorizedError(anyhow::anyhow!(
                        "Failed to find object_id {:?} when trying to create dynamic field info",
                        object_id
                    )),
                )?;
                let version = object.version().value();
                let digest = object.digest().to_string();
                let object_type = object.data.type_().unwrap().clone();
                DynamicFieldEntry {
                    parent_object_id: parent_id.to_string(),
                    transaction_digest: object.previous_transaction.base58_encode(),
                    checkpoint,
                    epoch,
                    timestamp_ms,
                    name: name_json,
                    bcs_name: Base64::encode(bcs_name),
                    type_,
                    object_id: object.id().to_string(),
                    digest,
                    version,
                    object_type: object_type.to_canonical_string(true),
                }
            }
        };
        self.dynamic_fields.push(entry);
        Ok(())
    }

    async fn process_transaction(
        &mut self,
        epoch: u64,
        checkpoint: u64,
        timestamp_ms: u64,
        checkpoint_transaction: &CheckpointTransaction,
    ) -> Result<()> {
        let all_objects: HashMap<_, _> = checkpoint_transaction
            .output_objects
            .iter()
            .map(|x| (x.id(), x.clone()))
            .collect();
        for object in checkpoint_transaction.output_objects.iter() {
            self.process_dynamic_field(epoch, checkpoint, timestamp_ms, object, &all_objects)
                .await?;
        }
        Ok(())
    }
}
