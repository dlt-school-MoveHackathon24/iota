// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use async_graphql::{connection::Connection, *};
use fastcrypto::encoding::{Base64, Encoding};
use iota_json_rpc_types::DevInspectArgs;
use iota_sdk::IotaClient;
use iota_types::{
    gas_coin::GAS,
    transaction::{TransactionData, TransactionDataAPI, TransactionKind},
    TypeTag,
};
use move_core_types::account_address::AccountAddress;
use serde::de::DeserializeOwned;

use super::{
    address::Address,
    available_range::AvailableRange,
    chain_identifier::ChainIdentifier,
    checkpoint::{self, Checkpoint, CheckpointId},
    coin::Coin,
    coin_metadata::CoinMetadata,
    cursor::Page,
    digest::Digest,
    dry_run_result::DryRunResult,
    epoch::Epoch,
    event::{self, Event, EventFilter},
    iota_address::IotaAddress,
    move_type::MoveType,
    object::{self, Object, ObjectFilter, ObjectLookupKey},
    owner::Owner,
    protocol_config::ProtocolConfigs,
    transaction_block::{self, TransactionBlock, TransactionBlockFilter},
    transaction_metadata::TransactionMetadata,
    type_filter::ExactTypeFilter,
};
use crate::{
    config::ServiceConfig,
    consistency::{consistent_range, CheckpointViewedAt},
    data::{Db, QueryExecutor},
    error::Error,
    mutation::Mutation,
    types::{
        base64::Base64 as GraphQLBase64,
        zklogin_verify_signature::{
            verify_zklogin_signature, ZkLoginIntentScope, ZkLoginVerifyResult,
        },
    },
};

pub(crate) struct Query;
pub(crate) type IotaGraphQLSchema = async_graphql::Schema<Query, Mutation, EmptySubscription>;

#[Object]
impl Query {
    /// First four bytes of the network's genesis checkpoint digest (uniquely
    /// identifies the network).
    async fn chain_identifier(&self, ctx: &Context<'_>) -> Result<String> {
        Ok(ChainIdentifier::query(ctx.data_unchecked())
            .await
            .extend()?
            .to_string())
    }

    /// Range of checkpoints that the RPC has data available for (for data
    /// that can be tied to a particular checkpoint).
    async fn available_range(&self, ctx: &Context<'_>) -> Result<AvailableRange> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;
        let result = ctx
            .data_unchecked::<Db>()
            .execute(move |conn| consistent_range(conn, Some(checkpoint_viewed_at)))
            .await
            .extend()?;

        match result {
            Some((first, last)) => Ok(AvailableRange { first, last }),
            None => Err(Error::Internal(
                "Checkpoint watermark outside of available range from database".to_string(),
            )
            .extend()),
        }
    }

    /// Configuration for this RPC service
    async fn service_config(&self, ctx: &Context<'_>) -> Result<ServiceConfig> {
        ctx.data()
            .map_err(|_| Error::Internal("Unable to fetch service configuration.".to_string()))
            .cloned()
            .extend()
    }

    /// Simulate running a transaction to inspect its effects without
    /// committing to them on-chain.
    ///
    /// `txBytes` either a `TransactionData` struct or a `TransactionKind`
    ///     struct, BCS-encoded and then Base64-encoded.  The expected
    ///     type is controlled by the presence or absence of `txMeta`: If
    ///     present, `txBytes` is assumed to be a `TransactionKind`, if
    ///     absent, then `TransactionData`.
    ///
    /// `txMeta` the data that is missing from a `TransactionKind` to make
    ///     a `TransactionData` (sender address and gas information).  All
    ///     its fields are nullable.
    ///
    /// `skipChecks` optional flag to disable the usual verification
    ///     checks that prevent access to objects that are owned by
    ///     addresses other than the sender, and calling non-public,
    ///     non-entry functions, and some other checks.  Defaults to false.
    async fn dry_run_transaction_block(
        &self,
        ctx: &Context<'_>,
        tx_bytes: String,
        tx_meta: Option<TransactionMetadata>,
        skip_checks: Option<bool>,
    ) -> Result<DryRunResult> {
        let skip_checks = skip_checks.unwrap_or(false);

        let iota_sdk_client: &Option<IotaClient> = ctx
            .data()
            .map_err(|_| Error::Internal("Unable to fetch Iota SDK client".to_string()))
            .extend()?;
        let iota_sdk_client = iota_sdk_client
            .as_ref()
            .ok_or_else(|| Error::Internal("Iota SDK client not initialized".to_string()))
            .extend()?;

        let (sender_address, tx_kind, gas_price, gas_sponsor, gas_budget, gas_objects) =
            if let Some(TransactionMetadata {
                sender,
                gas_price,
                gas_objects,
                gas_budget,
                gas_sponsor,
            }) = tx_meta
            {
                // This implies `TransactionKind`
                let tx_kind = deserialize_tx_data::<TransactionKind>(&tx_bytes)?;

                // Default is 0x0
                let sender_address = sender.unwrap_or_else(|| AccountAddress::ZERO.into()).into();

                let gas_sponsor = gas_sponsor.map(|addr| addr.into());

                let gas_objects = gas_objects.map(|objs| {
                    objs.into_iter()
                        .map(|obj| (obj.address.into(), obj.version.into(), obj.digest.into()))
                        .collect()
                });

                (
                    sender_address,
                    tx_kind,
                    gas_price.map(|p| p.into()),
                    gas_sponsor,
                    gas_budget.map(|b| b.into()),
                    gas_objects,
                )
            } else {
                // This implies `TransactionData`
                let tx_data = deserialize_tx_data::<TransactionData>(&tx_bytes)?;

                (
                    tx_data.sender(),
                    tx_data.clone().into_kind(),
                    Some(tx_data.gas_price().into()),
                    Some(tx_data.gas_owner()),
                    Some(tx_data.gas_budget().into()),
                    Some(tx_data.gas().to_vec()),
                )
            };

        let dev_inspect_args = DevInspectArgs {
            gas_sponsor,
            gas_budget,
            gas_objects,
            show_raw_txn_data_and_effects: Some(true),
            skip_checks: Some(skip_checks),
        };

        let res = iota_sdk_client
            .read_api()
            .dev_inspect_transaction_block(
                sender_address,
                tx_kind,
                gas_price,
                None,
                Some(dev_inspect_args),
            )
            .await?;

        DryRunResult::try_from(res).extend()
    }

    async fn owner(&self, ctx: &Context<'_>, address: IotaAddress) -> Result<Option<Owner>> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;

        Ok(Some(Owner {
            address,
            checkpoint_viewed_at: Some(checkpoint_viewed_at),
        }))
    }

    /// The object corresponding to the given address at the (optionally) given
    /// version. When no version is given, the latest version is returned.
    async fn object(
        &self,
        ctx: &Context<'_>,
        address: IotaAddress,
        version: Option<u64>,
    ) -> Result<Option<Object>> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;

        match version {
            Some(version) => Object::query(
                ctx.data_unchecked(),
                address,
                ObjectLookupKey::VersionAt {
                    version,
                    checkpoint_viewed_at: Some(checkpoint_viewed_at),
                },
            )
            .await
            .extend(),
            None => Object::query(
                ctx.data_unchecked(),
                address,
                ObjectLookupKey::LatestAt(checkpoint_viewed_at),
            )
            .await
            .extend(),
        }
    }

    /// Look-up an Account by its IotaAddress.
    async fn address(&self, ctx: &Context<'_>, address: IotaAddress) -> Result<Option<Address>> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;

        Ok(Some(Address {
            address,
            checkpoint_viewed_at: Some(checkpoint_viewed_at),
        }))
    }

    /// Fetch a structured representation of a concrete type, including its
    /// layout information. Fails if the type is malformed.
    async fn type_(&self, type_: String) -> Result<MoveType> {
        Ok(MoveType::new(
            TypeTag::from_str(&type_)
                .map_err(|e| Error::Client(format!("Bad type: {e}")))
                .extend()?,
        ))
    }

    /// Fetch epoch information by ID (defaults to the latest epoch).
    async fn epoch(&self, ctx: &Context<'_>, id: Option<u64>) -> Result<Option<Epoch>> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;

        Epoch::query(ctx, id, Some(checkpoint_viewed_at))
            .await
            .extend()
    }

    /// Fetch checkpoint information by sequence number or digest (defaults to
    /// the latest available checkpoint).
    async fn checkpoint(
        &self,
        ctx: &Context<'_>,
        id: Option<CheckpointId>,
    ) -> Result<Option<Checkpoint>> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;

        Checkpoint::query(ctx, id.unwrap_or_default(), Some(checkpoint_viewed_at))
            .await
            .extend()
    }

    /// Fetch a transaction block by its transaction digest.
    async fn transaction_block(
        &self,
        ctx: &Context<'_>,
        digest: Digest,
    ) -> Result<Option<TransactionBlock>> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;

        TransactionBlock::query(ctx.data_unchecked(), digest, Some(checkpoint_viewed_at))
            .await
            .extend()
    }

    /// The coin objects that exist in the network.
    ///
    /// The type field is a string of the inner type of the coin by which to
    /// filter (e.g. `0x2::iota::IOTA`). If no type is provided, it will
    /// default to `0x2::iota::IOTA`.
    async fn coins(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
        type_: Option<ExactTypeFilter>,
    ) -> Result<Connection<String, Coin>> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;

        let page = Page::from_params(ctx.data_unchecked(), first, after, last, before)?;
        let coin = type_.map_or_else(GAS::type_tag, |t| t.0);
        Coin::paginate(
            ctx.data_unchecked(),
            page,
            coin,
            // owner
            None,
            Some(checkpoint_viewed_at),
        )
        .await
        .extend()
    }

    /// The checkpoints that exist in the network.
    async fn checkpoints(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<checkpoint::Cursor>,
        last: Option<u64>,
        before: Option<checkpoint::Cursor>,
    ) -> Result<Connection<String, Checkpoint>> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;

        let page = Page::from_params(ctx.data_unchecked(), first, after, last, before)?;
        Checkpoint::paginate(
            ctx.data_unchecked(),
            page,
            // epoch
            None,
            Some(checkpoint_viewed_at),
        )
        .await
        .extend()
    }

    /// The transaction blocks that exist in the network.
    async fn transaction_blocks(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<transaction_block::Cursor>,
        last: Option<u64>,
        before: Option<transaction_block::Cursor>,
        filter: Option<TransactionBlockFilter>,
    ) -> Result<Connection<String, TransactionBlock>> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;

        let page = Page::from_params(ctx.data_unchecked(), first, after, last, before)?;
        TransactionBlock::paginate(
            ctx.data_unchecked(),
            page,
            filter.unwrap_or_default(),
            Some(checkpoint_viewed_at),
        )
        .await
        .extend()
    }

    /// The events that exist in the network.
    async fn events(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<event::Cursor>,
        last: Option<u64>,
        before: Option<event::Cursor>,
        filter: Option<EventFilter>,
    ) -> Result<Connection<String, Event>> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;

        let page = Page::from_params(ctx.data_unchecked(), first, after, last, before)?;
        Event::paginate(
            ctx.data_unchecked(),
            page,
            filter.unwrap_or_default(),
            Some(checkpoint_viewed_at),
        )
        .await
        .extend()
    }

    /// The objects that exist in the network.
    async fn objects(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
        filter: Option<ObjectFilter>,
    ) -> Result<Connection<String, Object>> {
        let CheckpointViewedAt(checkpoint_viewed_at) = *ctx.data()?;

        let page = Page::from_params(ctx.data_unchecked(), first, after, last, before)?;
        Object::paginate(
            ctx.data_unchecked(),
            page,
            filter.unwrap_or_default(),
            Some(checkpoint_viewed_at),
        )
        .await
        .extend()
    }

    /// Fetch the protocol config by protocol version (defaults to the latest
    /// protocol version known to the GraphQL service).
    async fn protocol_config(
        &self,
        ctx: &Context<'_>,
        protocol_version: Option<u64>,
    ) -> Result<ProtocolConfigs> {
        ProtocolConfigs::query(ctx.data_unchecked(), protocol_version)
            .await
            .extend()
    }

    /// The coin metadata associated with the given coin type.
    async fn coin_metadata(
        &self,
        ctx: &Context<'_>,
        coin_type: ExactTypeFilter,
    ) -> Result<Option<CoinMetadata>> {
        CoinMetadata::query(ctx.data_unchecked(), coin_type.0)
            .await
            .extend()
    }

    /// Verify a zkLogin signature based on the provided transaction or personal
    /// message based on current epoch, chain id, and latest JWKs fetched
    /// on-chain. If the signature is valid, the function returns a
    /// `ZkLoginVerifyResult` with success as true and an empty list of
    /// errors. If the signature is invalid, the function returns
    /// a `ZkLoginVerifyResult` with success as false with a list of errors.
    ///
    /// - `bytes` is either the personal message in raw bytes or transaction
    ///   data bytes in BCS-encoded and then Base64-encoded.
    /// - `signature` is a serialized zkLogin signature that is Base64-encoded.
    /// - `intentScope` is an enum that specifies the intent scope to be used to
    ///   parse bytes.
    /// - `author` is the address of the signer of the transaction or personal
    ///   msg.
    async fn verify_zklogin_signature(
        &self,
        ctx: &Context<'_>,
        bytes: GraphQLBase64,
        signature: GraphQLBase64,
        intent_scope: ZkLoginIntentScope,
        author: IotaAddress,
    ) -> Result<ZkLoginVerifyResult> {
        verify_zklogin_signature(ctx, bytes, signature, intent_scope, author)
            .await
            .extend()
    }
}

fn deserialize_tx_data<T>(tx_bytes: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    bcs::from_bytes(
        &Base64::decode(tx_bytes)
            .map_err(|e| {
                Error::Client(format!(
                    "Unable to deserialize transaction bytes from Base64: {e}"
                ))
            })
            .extend()?,
    )
    .map_err(|e| {
        Error::Client(format!(
            "Unable to deserialize transaction bytes as BCS: {e}"
        ))
    })
    .extend()
}
