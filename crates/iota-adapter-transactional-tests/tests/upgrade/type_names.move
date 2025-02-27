// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//# init --addresses A0=0x0 A1=0x0 A2=0x0 --accounts A

//# publish --upgradeable --sender A
module A0::m {
    public struct Canary has key {
        id: UID,
        addr: vector<u8>,
    }

    public struct A {}

}

//# upgrade --package A0 --upgrade-capability 1,0 --sender A
module A1::m {
    public struct Canary has key {
        id: UID,
        addr: vector<u8>,
    }

    public struct A {}
    public struct B {}
}

//# upgrade --package A1 --upgrade-capability 1,0 --sender A
module A2::m {
    use std::ascii;
    use std::type_name;

    public struct Canary has key {
        id: UID,
        addr: vector<u8>,
    }

    public struct A {}
    public struct B {}

    entry fun canary<T>(use_original: bool, ctx: &mut TxContext) {
        let type_ = if (use_original) {
            type_name::get_with_original_ids<T>()
        } else {
            type_name::get<T>()
        };

        let addr = ascii::into_bytes(type_name::get_address(&type_));

        transfer::transfer(
            Canary { id: object::new(ctx), addr },
            tx_context::sender(ctx),
        )
    }
}

//# run A2::m::canary --type-args A0::m::A --args true --sender A

//# run A2::m::canary --type-args A1::m::B --args true --sender A

//# run A2::m::canary --type-args A0::m::A --args false --sender A

//# run A2::m::canary --type-args A1::m::B --args false --sender A

//# view-object 4,0

//# view-object 5,0

//# view-object 6,0

//# view-object 7,0
