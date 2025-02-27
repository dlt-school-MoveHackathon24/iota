// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;

use move_core_types::{ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::{
    base_types::{IotaAddress, SequenceNumber},
    collection_types::{Bag, Table, VecSet},
    dynamic_field::get_dynamic_field_from_store,
    error::{UserInputError, UserInputResult},
    id::{ID, UID},
    object::{Object, Owner},
    storage::ObjectStore,
    IOTA_DENY_LIST_OBJECT_ID,
};

pub const DENY_LIST_MODULE: &IdentStr = ident_str!("deny_list");
pub const DENY_LIST_CREATE_FUNC: &IdentStr = ident_str!("create");

pub const DENY_LIST_COIN_TYPE_INDEX: u64 = 0;

/// Rust representation of the Move type 0x2::deny_list::DenyList.
/// It has a bag that contains the deny lists for different system types.
/// At creation, there is only one type (at key 0), which is the Coin type.
/// We also take advantage of the dynamic nature of Bag to add more types in the
/// future, as well as making changes to the deny lists for existing types.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DenyList {
    pub id: UID,
    pub lists: Bag,
}

/// Rust representation of the Move type 0x2::deny_list::PerTypeDenyList.
/// denied_count is a table that stores the number of denied addresses for each
/// coin template type. It can be used as a quick check to see if an address is
/// denied for any coin template type. denied_addresses is a table that stores
/// all the addresses that are denied for each coin template type. The key to
/// the table is the coin template type in string form:
/// package_id::module_name::struct_name.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerTypeDenyList {
    pub id: UID,
    // Table<address, u64>
    pub denied_count: Table,
    // Table<vec<u8>, VecSet<address>>
    pub denied_addresses: Table,
}

impl DenyList {
    pub fn check_coin_deny_list(
        address: IotaAddress,
        coin_types: BTreeSet<String>,
        object_store: &dyn ObjectStore,
    ) -> UserInputResult {
        let Some(deny_list) = get_coin_deny_list(object_store) else {
            // TODO: This is where we should fire an invariant violation metric.
            if cfg!(debug_assertions) {
                panic!("Failed to get the coin deny list");
            } else {
                return Ok(());
            }
        };
        Self::check_deny_list(deny_list, address, coin_types, object_store)
    }

    fn check_deny_list(
        deny_list: PerTypeDenyList,
        address: IotaAddress,
        coin_types: BTreeSet<String>,
        object_store: &dyn ObjectStore,
    ) -> UserInputResult {
        // TODO: Add caches to avoid repeated DF reads.
        let Ok(count) = get_dynamic_field_from_store::<IotaAddress, u64>(
            object_store,
            deny_list.denied_count.id,
            &address,
        ) else {
            return Ok(());
        };
        if count == 0 {
            return Ok(());
        }
        for coin_type in coin_types {
            let Ok(denied_addresses) = get_dynamic_field_from_store::<Vec<u8>, VecSet<IotaAddress>>(
                object_store,
                deny_list.denied_addresses.id,
                &coin_type.clone().into_bytes(),
            ) else {
                continue;
            };
            let denied_addresses: BTreeSet<_> = denied_addresses.contents.into_iter().collect();
            if denied_addresses.contains(&address) {
                debug!(
                    "Address {} is denied for coin package {:?}",
                    address, coin_type
                );
                return Err(UserInputError::AddressDeniedForCoin { address, coin_type });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinDenyCap {
    pub id: UID,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegulatedCoinMetadata {
    pub id: UID,
    pub coin_metadata_object: ID,
    pub deny_cap_object: ID,
}

pub fn get_deny_list_root_object(object_store: &dyn ObjectStore) -> Option<Object> {
    match object_store.get_object(&IOTA_DENY_LIST_OBJECT_ID) {
        Ok(Some(obj)) => Some(obj),
        Ok(None) => {
            error!("Deny list object not found");
            None
        }
        Err(err) => {
            error!("Failed to get deny list object: {}", err);
            None
        }
    }
}

pub fn get_coin_deny_list(object_store: &dyn ObjectStore) -> Option<PerTypeDenyList> {
    get_deny_list_root_object(object_store).and_then(|obj| {
        let deny_list: DenyList = obj
            .to_rust()
            .expect("DenyList object type must be consistent");
        match get_dynamic_field_from_store(
            object_store,
            *deny_list.lists.id.object_id(),
            &DENY_LIST_COIN_TYPE_INDEX,
        ) {
            Ok(deny_list) => Some(deny_list),
            Err(err) => {
                error!("Failed to get deny list inner state: {}", err);
                None
            }
        }
    })
}

pub fn get_deny_list_obj_initial_shared_version(
    object_store: &dyn ObjectStore,
) -> Option<SequenceNumber> {
    get_deny_list_root_object(object_store).map(|obj| match obj.owner {
        Owner::Shared {
            initial_shared_version,
        } => initial_shared_version,
        _ => unreachable!("Deny list object must be shared"),
    })
}
