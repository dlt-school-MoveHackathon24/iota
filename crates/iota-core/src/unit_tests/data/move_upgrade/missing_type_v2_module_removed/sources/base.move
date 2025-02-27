// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

module base_addr::base {

    struct A<T> {
        f1: bool,
        f2: T
    }

    public fun return_0(): u64 { abort 42 }

    public fun plus_1(x: u64): u64 { x + 1 }

    fun non_public_fun(y: bool): u64 { if (y) 0 else 1 }

    entry fun entry_fun() { }
}
