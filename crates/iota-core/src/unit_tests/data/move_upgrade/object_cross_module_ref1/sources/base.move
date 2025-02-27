// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

module base_addr::base {
    use iota::object::{Self, UID};
    use iota::tx_context::{Self, TxContext};
    use iota::transfer;
    use base_addr::friend_module::{Self, X, Z};

    struct A has store, drop {
        v: u16,
    }

    struct B has key {
        id: UID,
        field1: u32,
        field2: A,
    }

    struct C has key {
        id: UID,
        field1: u64,
        field2: X,
    }

    struct D has key {
        id: UID,
        field1: u64,
        field2: A,
    }

    struct E has key {
        id: UID,
        field1: u64,
        field2: X,
    }

    struct F has key {
        id: UID,
        field1: u64,
        field2: Z,
    }

    entry fun make_objs(ctx: &mut TxContext) {
        let field2 = A { v: 128 };
        transfer::transfer(
            B { id: object::new(ctx), field1: 256, field2 },
            tx_context::sender(ctx),
        );
        let field2 = friend_module::make_x(true);
        transfer::transfer(
            C { id: object::new(ctx), field1: 0, field2 },
            tx_context::sender(ctx),
        );
    }

    entry fun make_objs_v2(ctx: &mut TxContext) {
        let field2 = A { v: 128 };
        transfer::transfer(
            D { id: object::new(ctx), field1: 256, field2 },
            tx_context::sender(ctx),
        );
        let field2 = friend_module::make_x(true);
        transfer::transfer(
            E { id: object::new(ctx), field1: 0, field2 },
            tx_context::sender(ctx),
        );
        let field2 = friend_module::make_z(true);
        transfer::transfer(
            F { id: object::new(ctx), field1: 0, field2 },
            tx_context::sender(ctx),
        );
    }
}
