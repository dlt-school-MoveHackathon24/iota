// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use fastcrypto::encoding::Base64;
pub use fastcrypto::traits::KeyPair as KeypairTraits;
use iota_types::{
    base_types::{ObjectID, ObjectType},
    digests::{ObjectDigest, TransactionDigest},
    iota_serde::BigInt,
    move_package::{TypeOrigin, UpgradeInfo},
    object::{Object, Owner},
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct JsonObject {
    pub object_id: ObjectID,
    /// Object version.
    #[serde_as(as = "BigInt<u64>")]
    pub version: u64,
    /// Base64 string representing the object digest
    pub digest: ObjectDigest,
    /// The owner of this object.
    pub owner: Owner,
    /// The digest of the transaction that created or last mutated this object.
    /// Default to be None unless IotaObjectDataOptions.
    /// showPreviousTransaction is set to true
    pub previous_transaction: TransactionDigest,
    /// The amount of IOTA we would rebate if this object gets deleted.
    /// This number is re-calculated each time the object is mutated based on
    /// the present storage gas price.
    #[serde_as(as = "BigInt<u64>")]
    pub storage_rebate: u64,

    /// The type of the object
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "type")]
    pub type_: ObjectType,
    /// Move object content or package content
    pub data: JsonObjectData,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum JsonObjectData {
    Move(JsonMoveObject),
    Package(JsonPackage),
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct JsonPackage {
    // TODO do we want to dissasemble this?
    #[serde_as(as = "BTreeMap<_, Base64>")]
    module_map: BTreeMap<String, Vec<u8>>,

    /// Maps struct/module to a package version where it was first defined,
    /// stored as a vector for simple serialization and deserialization.
    type_origin_table: Vec<TypeOrigin>,

    // For each dependency, maps original package ID to the info about the (upgraded) dependency
    // version that this package is using
    linkage_table: BTreeMap<ObjectID, UpgradeInfo>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct JsonMoveObject {
    /// DEPRECATED this field is no longer used to determine whether a tx can
    /// transfer this object. Instead, it is always calculated from the
    /// objects type when loaded in execution
    has_public_transfer: bool,

    // TODO Do we want to spend the effort to render this?
    // Move Struct rendered with type info
    // fields: BTreeMap<String, IotaMoveValue>,
    #[serde_as(as = "Base64")]
    pub content: Vec<u8>,
}

impl JsonObject {
    pub fn from_object(object: &Object) -> Self {
        let (type_, object_data) = match &object.data {
            iota_types::object::Data::Move(struct_) => {
                let s = JsonMoveObject {
                    has_public_transfer: struct_.has_public_transfer(),
                    content: struct_.contents().to_vec(),
                };

                (
                    ObjectType::Struct(struct_.type_().clone()),
                    JsonObjectData::Move(s),
                )
            }
            iota_types::object::Data::Package(package) => {
                let pkg = JsonPackage {
                    module_map: package.serialized_module_map().clone(),
                    type_origin_table: package.type_origin_table().clone(),
                    linkage_table: package.linkage_table().clone(),
                };

                (ObjectType::Package, JsonObjectData::Package(pkg))
            }
        };

        let version = object.version().value();
        let object_id = object.id();

        Self {
            object_id,
            version,
            digest: object.digest(),
            owner: object.owner,
            previous_transaction: object.previous_transaction,
            storage_rebate: object.storage_rebate,
            type_,
            data: object_data,
        }
    }
}

/// Chain ID of the current chain
pub const X_IOTA_CHAIN_ID: &str = "x-iota-chain-id";

/// Current checkpoint height
pub const X_IOTA_CHECKPOINT_HEIGHT: &str = "x-iota-checkpoint-height";

/// Oldest non-pruned checkpoint height
pub const X_IOTA_OLDEST_CHECKPOINT_HEIGHT: &str = "x-iota-oldest-checkpoint-height";

/// Current epoch of the chain
pub const X_IOTA_EPOCH: &str = "x-iota-epoch";

/// Cursor to be used for endpoints that support cursor-based pagination. Pass
/// this to the start field of the endpoint on the next call to get the next
/// page of results.
pub const X_IOTA_CURSOR: &str = "x-iota-cursor";

/// Current timestamp of the chain - represented as number of milliseconds from
/// the Unix epoch
pub const X_IOTA_TIMESTAMP_MS: &str = "x-iota-timestamp-ms";
