// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use async_graphql::*;

use super::checkpoint::{Checkpoint, CheckpointId};

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub(crate) struct AvailableRange {
    pub first: u64,
    pub last: u64,
}

// TODO: do both in one query?
/// Range of checkpoints that the RPC is guaranteed to produce a consistent
/// response for.
#[Object]
impl AvailableRange {
    async fn first(&self, ctx: &Context<'_>) -> Result<Option<Checkpoint>> {
        Checkpoint::query(ctx, CheckpointId::by_seq_num(self.first), Some(self.last))
            .await
            .extend()
    }

    async fn last(&self, ctx: &Context<'_>) -> Result<Option<Checkpoint>> {
        Checkpoint::query(ctx, CheckpointId::by_seq_num(self.last), Some(self.last))
            .await
            .extend()
    }
}
