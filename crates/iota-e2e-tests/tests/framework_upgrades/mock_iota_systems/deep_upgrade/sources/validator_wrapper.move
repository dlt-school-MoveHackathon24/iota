// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

module iota_system::validator_wrapper {
    use iota::versioned::Versioned;
    use iota::versioned;
    use iota::tx_context::TxContext;
    use iota_system::validator::{Validator, ValidatorV2};
    use iota_system::validator;

    friend iota_system::iota_system;
    friend iota_system::iota_system_state_inner;

    const VALIDATOR_VERSION_V1: u64 = 18446744073709551605;  // u64::MAX - 10
    const VALIDATOR_VERSION_V3: u64 = 18446744073709551607;  // u64::MAX - 8

    const EInvalidVersion: u64 = 0;

    struct ValidatorWrapper has store {
        inner: Versioned
    }

    // Validator corresponds to version 1.
    public(friend) fun create_v1(validator: Validator, ctx: &mut TxContext): ValidatorWrapper {
        ValidatorWrapper {
            inner: versioned::create(VALIDATOR_VERSION_V1, validator, ctx)
        }
    }

    /// This function should always return the latest supported version.
    /// If the inner version is old, we upgrade it lazily in-place.
    public(friend) fun load_validator_maybe_upgrade(self: &mut ValidatorWrapper): &mut ValidatorV2 {
        upgrade_to_latest(self);
        versioned::load_value_mut(&mut self.inner)
    }

    /// Destroy the wrapper and retrieve the inner validator object.
    public(friend) fun destroy(self: ValidatorWrapper): ValidatorV2 {
        upgrade_to_latest(&mut self);
        let ValidatorWrapper { inner } = self;
        versioned::destroy(inner)
    }

    fun upgrade_to_latest(self: &mut ValidatorWrapper) {
        let version = version(self);
        if (version == VALIDATOR_VERSION_V1) {
            let (v1, cap) = versioned::remove_value_for_upgrade(&mut self.inner);
            let v3 = validator::v1_to_v2(v1);
            versioned::upgrade(&mut self.inner, VALIDATOR_VERSION_V3, v3, cap);
        };
        assert!(version(self) == VALIDATOR_VERSION_V3, EInvalidVersion);
    }

    fun version(self: &ValidatorWrapper): u64 {
        versioned::version(&self.inner)
    }
}
