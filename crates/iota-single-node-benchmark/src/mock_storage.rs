// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, sync::Arc};

use iota_storage::package_object_cache::PackageObjectCache;
use iota_types::{
    base_types::{EpochId, ObjectID, ObjectRef, SequenceNumber, VersionNumber},
    error::{IotaError, IotaResult},
    object::{Object, Owner},
    storage::{
        get_module_by_id, BackingPackageStore, ChildObjectResolver, GetSharedLocks, ObjectStore,
        PackageObject, ParentSync,
    },
    transaction::{InputObjectKind, InputObjects, ObjectReadResult, TransactionKey},
};
use move_binary_format::CompiledModule;
use move_bytecode_utils::module_cache::GetModule;
use move_core_types::language_storage::ModuleId;
use once_cell::unsync::OnceCell;
use prometheus::core::{Atomic, AtomicU64};

// TODO: We won't need a special purpose InMemoryObjectStore once the
// InMemoryCache is ready.
#[derive(Clone)]
pub(crate) struct InMemoryObjectStore {
    objects: Arc<HashMap<ObjectID, Object>>,
    package_cache: Arc<PackageObjectCache>,
    num_object_reads: Arc<AtomicU64>,
}

impl InMemoryObjectStore {
    pub(crate) fn new(objects: HashMap<ObjectID, Object>) -> Self {
        Self {
            objects: Arc::new(objects),
            package_cache: PackageObjectCache::new(),
            num_object_reads: Arc::new(AtomicU64::new(0)),
        }
    }

    pub(crate) fn get_num_object_reads(&self) -> u64 {
        self.num_object_reads.get()
    }

    // TODO: remove this when TransactionInputLoader is able to use the
    // ExecutionCache trait note: does not support shared object deletion.
    pub(crate) fn read_objects_for_execution(
        &self,
        shared_locks: &dyn GetSharedLocks,
        tx_key: &TransactionKey,
        input_object_kinds: &[InputObjectKind],
    ) -> IotaResult<InputObjects> {
        let shared_locks_cell: OnceCell<HashMap<_, _>> = OnceCell::new();
        let mut input_objects = Vec::new();
        for kind in input_object_kinds {
            let obj: Option<Object> = match kind {
                InputObjectKind::MovePackage(id) => self.get_package_object(id)?.map(|o| o.into()),
                InputObjectKind::ImmOrOwnedMoveObject(objref) => {
                    self.get_object_by_key(&objref.0, objref.1)?
                }

                InputObjectKind::SharedMoveObject { id, .. } => {
                    let shared_locks = shared_locks_cell.get_or_try_init(|| {
                        Ok::<HashMap<ObjectID, SequenceNumber>, IotaError>(
                            shared_locks.get_shared_locks(tx_key)?.into_iter().collect(),
                        )
                    })?;
                    let version = shared_locks.get(id).unwrap_or_else(|| {
                        panic!("Shared object locks should have been set. key: {tx_key:?}, obj id: {id:?}")
                    });

                    self.get_object_by_key(id, *version)?
                }
            };

            input_objects.push(ObjectReadResult::new(
                *kind,
                obj.ok_or_else(|| kind.object_not_found_error())?.into(),
            ));
        }

        Ok(input_objects.into())
    }
}

impl ObjectStore for InMemoryObjectStore {
    fn get_object(
        &self,
        object_id: &ObjectID,
    ) -> Result<Option<Object>, iota_types::storage::error::Error> {
        self.num_object_reads.inc_by(1);
        Ok(self.objects.get(object_id).cloned())
    }

    fn get_object_by_key(
        &self,
        object_id: &ObjectID,
        version: VersionNumber,
    ) -> Result<Option<Object>, iota_types::storage::error::Error> {
        Ok(self.get_object(object_id).unwrap().and_then(|o| {
            if o.version() == version {
                Some(o.clone())
            } else {
                None
            }
        }))
    }
}

impl BackingPackageStore for InMemoryObjectStore {
    fn get_package_object(&self, package_id: &ObjectID) -> IotaResult<Option<PackageObject>> {
        self.package_cache.get_package_object(package_id, self)
    }
}

impl ChildObjectResolver for InMemoryObjectStore {
    fn read_child_object(
        &self,
        parent: &ObjectID,
        child: &ObjectID,
        child_version_upper_bound: SequenceNumber,
    ) -> IotaResult<Option<Object>> {
        Ok(self.get_object(child).unwrap().and_then(|o| {
            if o.version() <= child_version_upper_bound
                && o.owner == Owner::ObjectOwner((*parent).into())
            {
                Some(o.clone())
            } else {
                None
            }
        }))
    }

    fn get_object_received_at_version(
        &self,
        _owner: &ObjectID,
        _receiving_object_id: &ObjectID,
        _receive_object_at_version: SequenceNumber,
        _epoch_id: EpochId,
    ) -> IotaResult<Option<Object>> {
        unimplemented!()
    }
}

impl GetModule for InMemoryObjectStore {
    type Error = IotaError;
    type Item = CompiledModule;

    fn get_module_by_id(&self, id: &ModuleId) -> Result<Option<Self::Item>, Self::Error> {
        get_module_by_id(self, id)
    }
}

impl ParentSync for InMemoryObjectStore {
    fn get_latest_parent_entry_ref_deprecated(
        &self,
        _object_id: ObjectID,
    ) -> IotaResult<Option<ObjectRef>> {
        unreachable!()
    }
}

impl GetSharedLocks for InMemoryObjectStore {
    fn get_shared_locks(
        &self,
        _key: &TransactionKey,
    ) -> Result<Vec<(ObjectID, SequenceNumber)>, IotaError> {
        unreachable!()
    }
}
