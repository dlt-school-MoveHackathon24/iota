// Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module timelock::test_label_two {

    use timelock::labels;

    /// Name of the label.
    public struct TEST_LABEL_TWO has drop {}

    /// Create and transfer a `LabelerCap` object to an authority address.
    public fun assign_labeler_cap(to: address, ctx: &mut TxContext) {
        // Fake one time witness.
        let witness = TEST_LABEL_TWO{};

        // Create a new capability.
        let cap = labels::create_labeler_cap<TEST_LABEL_TWO>(witness, ctx);

        // Transfer the capability to the specified address.
        transfer::public_transfer(cap, to);
    }
}
