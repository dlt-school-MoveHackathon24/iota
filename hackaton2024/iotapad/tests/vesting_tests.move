#[test_only]
module iotapad::vesting_test {
    use iota::coin::{Coin, TreasuryCap, mint_for_testing, create_treasury_cap_for_testing};
    use iota::clock::{Self};
    use iotapad::vesting::{Self, TimeFrame};

    // Struct to define the test type
    public struct VESTING_TEST has drop {}

    // Test case for the duration-based strategy
    #[test]
    public fun test_duration_based_releasable_amount() {
        let total_amount: u64 = 1000;
        let released_amount: u64 = 100;  // 100 tokens already released
        let start_time: u64 = 0;
        let vesting_duration: u64 = 3600;  // 1 hour duration

        // Test Case 1: Halfway through the duration
        let time_elapsed: u64 = 1800;  // 30 minutes have passed
        let releasable = vesting::get_frame_base_releasable_amount(
            total_amount,
            released_amount,
            start_time,
            time_elapsed,
            vesting_duration,
            true,  // This is a duration-based strategy
            vector::empty<TimeFrame>()  // Empty since it's duration-based
        );
        // Should release half of the remaining tokens (1000 * 1800 / 3600 = 500 total, minus 100 already released)
        assert!(releasable == 400, 100);

        // Test Case 2: Full duration passed
        let time_elapsed: u64 = 3600;  // 1 hour has passed
        let releasable = vesting::get_frame_base_releasable_amount(
            total_amount,
            released_amount,
            start_time,
            time_elapsed,
            vesting_duration,
            true,  // Duration-based strategy
            vector::empty<TimeFrame>()
        );
        // All remaining tokens should be released (1000 - 100 already released)
        assert!(releasable == 900, 101);

        // Test Case 3: More than the total duration passed
        let time_elapsed: u64 = 7200;  // 2 hours have passed
        let releasable = vesting::get_frame_base_releasable_amount(
            total_amount,
            released_amount,
            start_time,
            time_elapsed,
            vesting_duration,
            true,  // Duration-based strategy
            vector::empty<TimeFrame>()
        );
        // All remaining tokens should be released since the full duration has passed
        assert!(releasable == 900, 102);
    }
    // Test for creating a linear vesting strategy
    #[test]
    public fun test_create_linear_strategy() {
        let mut ctx = tx_context::dummy();  // Initialize a test context

        // Create a linear strategy (id_strategy = 1)
        let strategy = vesting::create_strategy_not_for_coin(1, 100, &mut ctx);
        let id_strategy = vesting::get_id_strategy(&strategy);
        let amount_each = vesting::get_amount_each(&strategy);

        // Verify the strategy was created successfully
        assert!(id_strategy == 1, 100);
        assert!(amount_each == 100, 101);
        
        transfer::public_transfer(strategy, tx_context::sender(&ctx));
    }

    // Test for creating a dynamic vesting strategy
    #[test]
    public fun test_create_dynamic_strategy() {
        let mut ctx = tx_context::dummy();  // Initialize a test context

        // Create a dynamic strategy (id_strategy = 2)
        let strategy = vesting::create_strategy_not_for_coin(2, 50, &mut ctx);
        let id_strategy = vesting::get_id_strategy(&strategy);
        let amount_each = vesting::get_amount_each(&strategy);

        // Verify the strategy was created successfully
        assert!(id_strategy == 2, 200);
        assert!(amount_each == 50, 201);
        transfer::public_transfer(strategy, tx_context::sender(&ctx));
    }

    // Test for creating a linear vesting contract with duration
    #[test]
    public fun test_create_linear_vesting_with_duration() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&clock);

        // Create a linear strategy
        let strategy = vesting::create_strategy_not_for_coin(1, 100, &mut ctx);

        // No time frames, using duration-based vesting
        let duration_seconds = option::some<u64>(3600);  // 1 hour duration
        let vesting_contract = vesting::create_vester<VESTING_TEST, u64>(
            start_time,
            strategy,
            duration_seconds,
            option::none(),
            option::none(),
            &mut ctx
        );

        // Check that the duration is set correctly
        let duration = vesting::get_duration(&vesting_contract);
        assert!(duration == option::some<u64>(3600), 301);

        clock::destroy_for_testing(clock);
        transfer::public_transfer(vesting_contract, tx_context::sender(&ctx));
    }

    // Test for creating a dynamic vesting contract with time frames
    #[test]
    public fun test_create_dynamic_vesting_with_timeframes() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&clock);

        // Create a dynamic strategy
        let strategy = vesting::create_strategy_not_for_coin(2, 50, &mut ctx);

        // Create time frames and percentages
        let times = option::some<vector<u64>>(vector[start_time + 1800, start_time + 3600]);  // Two time frames
        let percentages = option::some<vector<u8>>(vector[50, 50]);  // 50%, 50%

        // Create vesting contract with time frames
        let vesting_contract = vesting::create_vester<VESTING_TEST, u64>(
            start_time,
            strategy,
            option::none(),
            times,
            percentages,
            &mut ctx
        );

        // Check that the time frames are set correctly
        let time_frames = vesting::get_time_frames(&vesting_contract);
        assert!(option::is_some(&time_frames), 400);

        clock::destroy_for_testing(clock);
        transfer::public_transfer(vesting_contract, tx_context::sender(&ctx));
    }

    // Test for releasing coins from a linear vesting contract
    #[test]
    public fun test_release_coins_linear_vesting() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let mut clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&clock);

        // Create a linear vesting strategy
        let strategy = vesting::create_strategy_not_for_coin(1, 100, &mut ctx);
        let duration_seconds = option::some<u64>(3600);  // 1 hour duration

        // Create vesting contract
        let mut vesting_contract = vesting::create_vester<VESTING_TEST, u64>(
            start_time,
            strategy,
            duration_seconds,
            option::none(),
            option::none(),
            &mut ctx
        );

        // Add funds to the vesting contract
        let to_vest:Coin<VESTING_TEST> = mint_for_testing<VESTING_TEST>(1000, &mut ctx);
        vesting::initialize_vester_with_coin<VESTING_TEST, u64>(&mut vesting_contract, to_vest, &mut ctx);

        // Advance time by 30 minutes
        clock::increment_for_testing(&mut clock, 1800);

        // Attempt to release coins before 1 hour (should fail)
        vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut ctx);

        // Advance time to 1 hour and release coins
        clock::increment_for_testing(&mut clock, 1800);
        vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut ctx);

        assert!(vesting::get_total_vested(&vesting_contract) == 100, 501);  // Check that coins have been released

        clock::destroy_for_testing(clock);
        transfer::public_transfer(vesting_contract, tx_context::sender(&ctx));
    }

    // Test for releasing coins from a time-frame-based vesting contract
    #[test]
    //#[expected_failure(abort_code = vesting::ERROR_INSUFFICIENT_FUNDS)] // clock time is not progressing
    public fun test_release_coins_timeframes() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let mut clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&clock);

        // Create a dynamic vesting strategy
        let strategy = vesting::create_strategy_not_for_coin(2, 50, &mut ctx);

        // Create time frames and percentages
        let times = option::some<vector<u64>>(vector[start_time + 1800, start_time + 3600]);  // Two time frames
        let percentages = option::some<vector<u8>>(vector[50, 50]);  // 50%, 50%

        // Create vesting contract with time frames
        let mut vesting_contract = vesting::create_vester<VESTING_TEST, u64>(
            start_time,
            strategy,
            option::none(),
            times,
            percentages,
            &mut ctx
        );

        let mut a_user1 = tx_context::new_from_hint(
            tx_context::fresh_object_address(&mut ctx), 1, 1, 1, 1
        );


        let mut a_user2 = tx_context::new_from_hint(
            tx_context::fresh_object_address(&mut ctx), 2, 2, 2, 2
        );

        // Add funds to the vesting contract
        let to_vest = mint_for_testing<VESTING_TEST>(1000, &mut ctx);
        vesting::initialize_vester_with_coin<VESTING_TEST, u64>(&mut vesting_contract, to_vest, &mut ctx);

        vesting::set_allocate_amount_per_user(
            &mut vesting_contract, 
            vector[tx_context::sender(&ctx), tx_context::sender(&a_user1),  tx_context::sender(&a_user2)], 
            vector[100, 50, 50], 
            &mut ctx);

        // Advance time by 30 minutes and release 50%
        clock::increment_for_testing(&mut clock, 2000);
        vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut ctx);
        vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut a_user1);


        assert!(vesting::t_get_released_amount_to_user(&mut vesting_contract, tx_context::sender(&ctx))== 50, 600);  // 50% released
        assert!(vesting::t_get_released_amount_to_user(&mut vesting_contract, tx_context::sender(&a_user1))== 25, 600);  // 50% released
        assert!(vesting::get_total_vested(&vesting_contract)== 75, 600);  // 50% released

        // Advance time to 1 hour and release remaining 50%
        clock::increment_for_testing(&mut clock, 2000);
        vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut ctx);
        vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut a_user1);
        vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut a_user2);

        assert!(vesting::get_total_vested(&vesting_contract) == 200, 601);  // Remaining 50% released

        clock::destroy_for_testing(clock);
        transfer::public_transfer(vesting_contract, tx_context::sender(&ctx));
    }

    // Test for adding supply to the vesting contract
    #[test]
    public fun test_add_supply() {
        let mut ctx = tx_context::dummy();  // Initialize a test context

        // Create a linear vesting strategy
        let strategy = vesting::create_strategy_not_for_coin(1, 100, &mut ctx);
        let duration_seconds = option::some<u64>(3600);  // 1 hour duration
        let clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&clock);

        // Create vesting contract
        let mut vesting_contract = vesting::create_vester<VESTING_TEST, u64>(
            start_time,
            strategy,
            duration_seconds,
            option::none(),
            option::none(),
            &mut ctx
        );

        // Add supply to the vesting contract
        let additional_supply = mint_for_testing<VESTING_TEST>(500, &mut ctx);
        vesting::add_supply_of_coin<VESTING_TEST, u64>(
            &mut vesting_contract,
            additional_supply,
            &mut ctx
        );

        // Check that the supply is updated correctly
        let supply = vesting::get_supply(&vesting_contract);
        assert!(supply == 500, 700);

        clock::destroy_for_testing(clock);
        transfer::public_transfer(vesting_contract, tx_context::sender(&ctx));
    }

    // Test for handling invalid timeframes and percentages
    #[test]
    #[expected_failure(abort_code = vesting::ERROR_INVALID_TIME_FRAME_PARAMETERS)]
    public fun test_invalid_timeframes_and_percentages() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&clock);

        // Create a dynamic strategy
        let strategy = vesting::create_strategy_not_for_coin(2, 50, &mut ctx);

        // Create mismatched time frames and percentages (should fail)
        let times = option::some<vector<u64>>(vector[start_time + 1800]);  // One time frame
        let percentages = option::some<vector<u8>>(vector[50, 50]);  // Two percentages (mismatch)

        // Attempt to create vesting contract (should fail)
        let vesting_contract = vesting::create_vester<VESTING_TEST, u64>(
            start_time,
            strategy,
            option::none(),
            times,
            percentages,
            &mut ctx
        );
        
        clock::destroy_for_testing(clock);
        transfer::public_transfer(vesting_contract, tx_context::sender(&ctx));
    }

    // Test for collecting non-vested coins
    #[test]
    public fun test_collect_not_vested_coins() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let mut clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&clock);

        // Create a linear vesting strategy
        let strategy = vesting::create_strategy_not_for_coin(1, 100, &mut ctx);
        let duration_seconds = option::some<u64>(3600);  // 1 hour duration

        // Create vesting contract
        let mut vesting_contract = vesting::create_vester<VESTING_TEST, u64>(
            start_time,
            strategy,
            duration_seconds,
            option::none(),
            option::none(),
            &mut ctx
        );

        // Add supply to the vesting contract
        let to_vest = mint_for_testing<VESTING_TEST>(1000, &mut ctx);
        vesting::initialize_vester_with_coin<VESTING_TEST, u64>(&mut vesting_contract, to_vest, &mut ctx);

        // Advance time to after the vesting period
        clock::increment_for_testing(&mut clock, 3600);  // Move time 1 hour forward

        //Collect non-vested coins after vesting ends (should succeed)
        vesting::collect_not_vested_coins<VESTING_TEST, u64>(&mut vesting_contract, &clock, &mut ctx);

        // Verify that all coins are collected, so the contract should have zero balance
        let balance = vesting::get_balance(&vesting_contract);
        assert!(balance == 0, 101);

        clock::destroy_for_testing(clock);
        transfer::public_transfer(vesting_contract, tx_context::sender(&ctx));
    }

    // Test for releasing coins via coin-based strategy
    #[test]
    public fun test_release_coins_by_coinbase() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let mut clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&clock);

        // Create a coin-based strategy
        let strategy = vesting::create_strategy_for_coin<VESTING_TEST>(3, 100, &mut ctx);

        // Create time frames and percentages for release
        let times = option::some<vector<u64>>(vector[start_time + 1800, start_time + 3600]);  // Two time frames
        let percentages = option::some<vector<u8>>(vector[50, 50]);  // 50% at each time frame

        // Create a vesting contract using the coin-based strategy
        let mut vesting_contract = vesting::create_vester<VESTING_TEST, VESTING_TEST>(
            start_time,
            strategy,
            option::none<u64>(),  // No duration-based vesting
            times,
            percentages,
            &mut ctx
        );

        // Mint some "BaseCoin" to simulate user's coin holdings
        let user_coins = mint_for_testing<VESTING_TEST>(1000, &mut ctx);
        let coin_list = vector::singleton(user_coins);

        // Add supply to the vesting contract
        let to_vest = mint_for_testing<VESTING_TEST>(1000, &mut ctx);
        vesting::initialize_vester_with_coin<VESTING_TEST, VESTING_TEST>(&mut vesting_contract, to_vest, &mut ctx);


        // Advance time to after the first time frame (50% releasable)
        clock::increment_for_testing(&mut clock, 1800);

        // Test Case 2: Release 50% of the coins after the first time frame
        vesting::release_coins_by_coinbase<VESTING_TEST, VESTING_TEST>(
            &mut vesting_contract,
            &clock,
            coin_list,
            &mut ctx
        );

        // Verify that 50% (500 coins) were released
        let total_vested = vesting::get_total_vested(&vesting_contract);
        assert!(total_vested == 500, 201);

        // Advance time to after the second time frame (100% releasable)
        clock::increment_for_testing(&mut clock, 1800);

        
        let user_coins = mint_for_testing<VESTING_TEST>(0, &mut ctx);
        let coin_list = vector::singleton(user_coins);

        // Test Case 3: Release the remaining 50% of the coins after the second time frame
        vesting::release_coins_by_coinbase<VESTING_TEST, VESTING_TEST>(
            &mut vesting_contract,
            &clock,
            coin_list,
            &mut ctx
        );

        // Verify that 100% of the coins (1000 coins) are released
        let total_vested = vesting::get_total_vested(&vesting_contract);
        assert!(total_vested == 1000, 202);

        clock::destroy_for_testing(clock);
        transfer::public_transfer(vesting_contract, tx_context::sender(&ctx));
    }

    #[test]
    public fun test_mint_and_vest_success() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let mut clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&clock);

        // Create a linear vesting strategy
        let strategy = vesting::create_strategy_not_for_coin(1, 100, &mut ctx);

        // Set a duration of 1 hour
        let duration_seconds = option::some<u64>(3600);

        // Create the vesting contract with the mintable flag set to true
        let mut vesting_contract = vesting::create_vester<VESTING_TEST, u64>(
            start_time,
            strategy,
            duration_seconds,
            option::none(),
            option::none(),
            &mut ctx
        );

        // Vesting contract using a TreasuryCap
        let treasury_cap = create_treasury_cap_for_testing<VESTING_TEST>(&mut ctx);  // Mintable treasury cap
        let mut data: Option<TreasuryCap<VESTING_TEST>>  = vesting::set_mintable_treasury(&mut vesting_contract, treasury_cap, &mut ctx);

        // Advance time by 30 minutes
        clock::increment_for_testing(&mut clock, 1800);

        // Try to release tokens (50% should be releasable)
        vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut ctx);

        // Check that 500 tokens have been vested and minted
        assert!(vesting::get_total_vested(&vesting_contract) == 50, 701);

        // Advance time to 1 hour and release the remaining tokens
        clock::increment_for_testing(&mut clock, 1800);
        vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut ctx);

        // Check that all 1000 tokens have been vested
        assert!(vesting::get_total_vested(&vesting_contract) == 100, 702);

        clock::destroy_for_testing(clock);
        transfer::public_transfer(vesting_contract, tx_context::sender(&ctx));
        if (option::is_some(&data)) {
            let treasury : TreasuryCap<VESTING_TEST> = option::extract(&mut data);
            transfer::public_transfer(treasury, tx_context::sender(&ctx));
        };

        option::destroy_none(data);
    }

    #[test]
    #[expected_failure(abort_code = vesting::ERROR_INSUFFICIENT_FUNDS)]
    public fun test_mint_and_vest_fail_without_treasury() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let mut clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&clock);

        // Create a linear vesting strategy
        let strategy = vesting::create_strategy_not_for_coin(1, 100, &mut ctx);

        // Set a duration of 1 hour
        let duration_seconds = option::some<u64>(3600);

        // Create the vesting contract with the mintable flag set to true but without treasury minting capability
        let mut vesting_contract = vesting::create_vester<VESTING_TEST, u64>(
            start_time,
            strategy,
            duration_seconds,
            option::none(),
            option::none(),
            &mut ctx
        );

        // Advance time to 1 hour and release the remaining tokens
        clock::increment_for_testing(&mut clock, 3600);
        // Attempt to release tokens (should fail due to missing treasury minting permissions)
        vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut ctx);

        clock::destroy_for_testing(clock);
        transfer::public_transfer(vesting_contract, tx_context::sender(&ctx));
        
    }

}
