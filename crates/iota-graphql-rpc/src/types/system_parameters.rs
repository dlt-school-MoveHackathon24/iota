// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use async_graphql::*;

use super::big_int::BigInt;

/// Details of the system that are decided during genesis.
#[derive(Clone, Debug, PartialEq, Eq, SimpleObject)]
pub(crate) struct SystemParameters {
    /// Target duration of an epoch, in milliseconds.
    pub duration_ms: Option<BigInt>,

    /// The minimum number of active validators that the system supports.
    pub min_validator_count: Option<u64>,

    /// The maximum number of active validators that the system supports.
    pub max_validator_count: Option<u64>,

    /// Minimum stake needed to become a new validator.
    pub min_validator_joining_stake: Option<BigInt>,

    /// Validators with stake below this threshold will enter the grace period
    /// (see `validatorLowStakeGracePeriod`), after which they are removed
    /// from the active validator set.
    pub validator_low_stake_threshold: Option<BigInt>,

    /// Validators with stake below this threshold will be removed from the
    /// active validator set at the next epoch boundary, without a grace
    /// period.
    pub validator_very_low_stake_threshold: Option<BigInt>,

    /// The number of epochs that a validator has to recover from having less
    /// than `validatorLowStakeThreshold` stake.
    pub validator_low_stake_grace_period: Option<BigInt>,
}
