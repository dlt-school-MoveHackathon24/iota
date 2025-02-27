// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use async_graphql::*;

use super::gas::GasCostSummary;

/// Information about whether epoch changes are using safe mode.
#[derive(Clone, Debug, PartialEq, Eq, SimpleObject)]
pub(crate) struct SafeMode {
    /// Whether safe mode was used for the last epoch change.  The system will
    /// retry a full epoch change on every epoch boundary and automatically
    /// reset this flag if so.
    pub enabled: Option<bool>,

    /// Accumulated fees for computation and cost that have not been added to
    /// the various reward pools, because the full epoch change did not
    /// happen.
    pub gas_summary: Option<GasCostSummary>,
}
