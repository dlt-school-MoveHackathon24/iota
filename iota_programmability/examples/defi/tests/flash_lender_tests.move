// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module defi::flash_lender_tests {
    use defi::flash_lender::{Self, AdminCap, FlashLender};
    use iota::pay;
    use iota::coin;
    use iota::iota::IOTA;
    use iota::test_scenario;

    #[test]
    fun flash_loan_example() {
        let admin = @0x1;
        let borrower = @0x2;

        // admin creates a flash lender with 100 coins and a fee of 1 coin
        let mut scenario_val = test_scenario::begin(admin);
        let scenario = &mut scenario_val;
        {
            let ctx = test_scenario::ctx(scenario);
            let coin = coin::mint_for_testing<IOTA>(100, ctx);
            flash_lender::create(coin, 1, ctx);
        };
        // borrower requests and repays a loan of 10 coins + the fee
        test_scenario::next_tx(scenario, borrower);
        {
            let mut lender_val = test_scenario::take_shared<FlashLender<IOTA>>(scenario);
            let lender = &mut lender_val;
            let ctx = test_scenario::ctx(scenario);

            let (loan, receipt) = flash_lender::loan(lender, 10, ctx);
            // in practice, borrower does something (e.g., arbitrage) to make a profit from the loan.
            // simulate this by minting the borrower 5 coins.
            let mut profit = coin::mint_for_testing<IOTA>(5, ctx);
            coin::join(&mut profit, loan);
            let to_keep = coin::take(coin::balance_mut(&mut profit), 4, ctx);
            pay::keep(to_keep, ctx);
            flash_lender::repay(lender, profit, receipt);

            test_scenario::return_shared(lender_val);
        };
        // admin withdraws the 1 coin profit from lending
        test_scenario::next_tx(scenario, admin);
        {
            let mut lender_val = test_scenario::take_shared<FlashLender<IOTA>>(scenario);
            let lender = &mut lender_val;
            let admin_cap = test_scenario::take_from_sender<AdminCap>(scenario);
            let ctx = test_scenario::ctx(scenario);

            // max loan size should have increased because of the fee payment
            assert!(flash_lender::max_loan(lender) == 101, 0);
            // withdraw 1 coin from the pool available for lending
            let coin = flash_lender::withdraw(lender, &admin_cap, 1, ctx);
            // max loan size should decrease accordingly
            assert!(flash_lender::max_loan(lender) == 100, 0);
            pay::keep(coin, ctx);

            test_scenario::return_shared(lender_val);
            test_scenario::return_to_sender(scenario, admin_cap);
        };
        test_scenario::end(scenario_val);
    }
}
