// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

module base_addr::friend_module {

    struct X has store, drop {
        v: bool,
    }

    struct Y has store, drop {
        v: u64,
    }

    public fun make_x(v: bool): X {
        X { v }
    }

    public fun make_y(v: u64): Y {
        Y { v }
    }
}
