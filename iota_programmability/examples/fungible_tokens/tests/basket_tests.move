// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module fungible_tokens::basket_tests {
    use fungible_tokens::basket::{Self, Reserve};
    use fungible_tokens::managed::MANAGED;
    use iota::pay;
    use iota::coin;
    use iota::iota::IOTA;
    use iota::test_scenario;

    #[test]
    public fun test_mint_burn() {
        let user = @0xA;

        let mut scenario_val = test_scenario::begin(user);
        let scenario = &mut scenario_val;
        {
            let ctx = test_scenario::ctx(scenario);
            basket::init_for_testing(ctx);
        };
        test_scenario::next_tx(scenario, user);
        {
            let mut reserve_val = test_scenario::take_shared<Reserve>(scenario);
            let reserve = &mut reserve_val;
            let ctx = test_scenario::ctx(scenario);
            assert!(basket::total_supply(reserve) == 0, 0);

            let num_coins = 10;
            let iota = coin::mint_for_testing<IOTA>(num_coins, ctx);
            let managed = coin::mint_for_testing<MANAGED>(num_coins, ctx);
            let basket = basket::mint(reserve, iota, managed, ctx);
            assert!(coin::value(&basket) == num_coins, 1);
            assert!(basket::total_supply(reserve) == num_coins, 2);

            let (iota, managed) = basket::burn(reserve, basket, ctx);
            assert!(coin::value(&iota) == num_coins, 3);
            assert!(coin::value(&managed) == num_coins, 4);

            pay::keep(iota, ctx);
            pay::keep(managed, ctx);
            test_scenario::return_shared(reserve_val);
        };
        test_scenario::end(scenario_val);
    }

}
