// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};

use iota_storage::blob::{Blob, BlobEncoding};
use iota_types::{
    digests::TransactionDigest,
    effects::TransactionEffectsAPI,
    error::{IotaError, IotaResult, UserInputError},
    full_checkpoint_content::{CheckpointData, CheckpointTransaction},
    messages_checkpoint::VerifiedCheckpoint,
    storage::ObjectKey,
};

use crate::{checkpoints::CheckpointStore, execution_cache::ExecutionCacheRead};

pub(crate) fn store_checkpoint_locally(
    path: PathBuf,
    checkpoint: VerifiedCheckpoint,
    cache_reader: &dyn ExecutionCacheRead,
    checkpoint_store: Arc<CheckpointStore>,
    transaction_digests: Vec<TransactionDigest>,
) -> IotaResult {
    let checkpoint_contents = checkpoint_store
        .get_checkpoint_contents(&checkpoint.content_digest)?
        .expect("checkpoint content has to be stored");

    let transactions = cache_reader
        .multi_get_transaction_blocks(&transaction_digests)?
        .into_iter()
        .zip(&transaction_digests)
        .map(|(tx, digest)| tx.ok_or(IotaError::TransactionNotFound { digest: *digest }))
        .collect::<IotaResult<Vec<_>>>()?;

    let effects = cache_reader
        .multi_get_executed_effects(&transaction_digests)?
        .into_iter()
        .zip(transaction_digests)
        .map(|(effects, digest)| effects.ok_or(IotaError::TransactionNotFound { digest }))
        .collect::<IotaResult<Vec<_>>>()?;

    let event_digests = effects
        .iter()
        .flat_map(|fx| fx.events_digest().copied())
        .collect::<Vec<_>>();

    let events = cache_reader
        .multi_get_events(&event_digests)?
        .into_iter()
        .zip(&event_digests)
        .map(|(event, digest)| {
            event.ok_or(IotaError::TransactionEventsNotFound { digest: *digest })
        })
        .collect::<IotaResult<Vec<_>>>()?;

    let events: HashMap<_, _> = event_digests.into_iter().zip(events).collect();
    let mut full_transactions = Vec::with_capacity(transactions.len());
    for (tx, fx) in transactions.into_iter().zip(effects) {
        let events = fx.events_digest().map(|event_digest| {
            events
                .get(event_digest)
                .cloned()
                .expect("event was already checked to be present")
        });
        // Note unwrapped_then_deleted contains **updated** versions.
        let unwrapped_then_deleted_obj_ids = fx
            .unwrapped_then_deleted()
            .into_iter()
            .map(|k| k.0)
            .collect::<HashSet<_>>();

        let input_object_keys = fx
            .input_shared_objects()
            .into_iter()
            .map(|kind| {
                let (id, version) = kind.id_and_version();
                ObjectKey(id, version)
            })
            .chain(
                fx.modified_at_versions()
                    .into_iter()
                    .map(|(object_id, version)| ObjectKey(object_id, version)),
            )
            .collect::<HashSet<_>>()
            .into_iter()
            // Unwrapped-then-deleted objects are not stored in state before the tx, so we have
            // nothing to fetch.
            .filter(|key| !unwrapped_then_deleted_obj_ids.contains(&key.0))
            .collect::<Vec<_>>();

        let input_objects = cache_reader
            .multi_get_objects_by_key(&input_object_keys)?
            .into_iter()
            .zip(&input_object_keys)
            .map(|(object, object_key)| {
                object.ok_or(IotaError::UserInput {
                    error: UserInputError::ObjectNotFound {
                        object_id: object_key.0,
                        version: Some(object_key.1),
                    },
                })
            })
            .collect::<IotaResult<Vec<_>>>()?;

        let output_object_keys = fx
            .all_changed_objects()
            .into_iter()
            .map(|(object_ref, _owner, _kind)| ObjectKey::from(object_ref))
            .collect::<Vec<_>>();

        let output_objects = cache_reader
            .multi_get_objects_by_key(&output_object_keys)?
            .into_iter()
            .zip(&output_object_keys)
            .map(|(object, object_key)| {
                object.ok_or(IotaError::UserInput {
                    error: UserInputError::ObjectNotFound {
                        object_id: object_key.0,
                        version: Some(object_key.1),
                    },
                })
            })
            .collect::<IotaResult<Vec<_>>>()?;

        let full_transaction = CheckpointTransaction {
            transaction: (*tx).clone().into(),
            effects: fx,
            events,
            input_objects,
            output_objects,
        };
        full_transactions.push(full_transaction);
    }
    let file_name = format!("{}.chk", checkpoint.sequence_number);
    let checkpoint_data = CheckpointData {
        checkpoint_summary: checkpoint.into(),
        checkpoint_contents,
        transactions: full_transactions,
    };

    std::fs::create_dir_all(&path).map_err(|err| {
        IotaError::FileIO(format!(
            "failed to save full checkpoint content locally {:?}",
            err
        ))
    })?;

    Blob::encode(&checkpoint_data, BlobEncoding::Bcs)
        .map_err(|_| IotaError::TransactionSerialization {
            error: "failed to serialize full checkpoint content".to_string(),
        }) // Map the first error
        .and_then(|blob| {
            std::fs::write(path.join(file_name), blob.to_bytes()).map_err(|_| {
                IotaError::FileIO("failed to save full checkpoint content locally".to_string())
            })
        })?;

    Ok(())
}
