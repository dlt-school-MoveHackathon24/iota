// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

module base_addr::base {
    use iota::object::UID;

    struct A<T> {
        f1: bool,
        f2: T
    }

    struct B has key {
        id: UID,
        x: u64,
    }

    friend base_addr::friend_module;


    public fun return_0(): u64 { abort 42 }

    public fun plus_1(x: u64): u64 { x + 1 }

    public(friend) fun friend_fun(x: u64): u64 { x }

    fun non_public_fun(y: bool): u64 { if (y) 0 else 1 }
}
