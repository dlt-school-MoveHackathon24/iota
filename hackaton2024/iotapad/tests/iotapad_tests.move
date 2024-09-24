#[test_only]
module iotapad::iotapad_tests {
    use iotapad::iotapad;
    use iota::coin::{Coin, mint_for_testing};
    use iota::clock::{Self};

    public struct IOTAPAD_TESTS has drop {}

    #[test]
    public fun test_create_launchpad_valid() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let c_clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&c_clock);
        let end_time = start_time + 10000;           // End time is 10 seconds later
        let times = vector::singleton(start_time + 5000); // Vesting time 5 seconds after start
        let percentages = vector::singleton(100);    // Full vesting (100%) at the given time

        // Create the Launchpad
        iotapad::create_launchpad<Coin<IOTAPAD_TESTS>, Coin<IOTAPAD_TESTS>>(
            1,
            10,  // Conversion rate
            1000,  // Total allocation
            start_time,
            end_time,
            tx_context::sender(&ctx),
            times,
            percentages,
            &mut ctx
        );
        
        clock::destroy_for_testing(c_clock);
    }

    #[test]
    #[expected_failure(abort_code = iotapad::ERROR_INVALID_TIME_FRAME_PARAMETERS)]
    public fun test_create_launchpad_invalid_parameters() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let c_clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&c_clock);
        let end_time = start_time + 10000;

        let times = vector::singleton(start_time + 5000); // One time frame
        let percentages = vector::empty<u8>();            // No percentages (invalid)

        iotapad::create_launchpad<Coin<IOTAPAD_TESTS>, Coin<IOTAPAD_TESTS>>(
            1,
                10,
                1000,
                start_time,
                end_time,
                tx_context::sender(&ctx),
                times,
                percentages,
                &mut ctx
            );
        clock::destroy_for_testing(c_clock);
            
    }

    #[test]
    public fun test_register_launchpad_success() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let staked_amount: Coin<IOTAPAD_TESTS> = mint_for_testing<IOTAPAD_TESTS>(1000, &mut ctx);  // Mint some tokens for staking

        // Create a valid launchpad first
        let c_clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&c_clock);
        let end_time = start_time + 10000;
        let times = vector::singleton(start_time + 5000);
        let percentages = vector::singleton(100);

        let mut launchpad = iotapad::get_create_launchpad<IOTAPAD_TESTS, IOTAPAD_TESTS>(
            1,
            10,
            1000,
            start_time,
            end_time,
            tx_context::sender(&ctx),
            times,
            percentages,
            &mut ctx
        );

        let having_amount = iotapad::register_launchpad<IOTAPAD_TESTS, IOTAPAD_TESTS>(&mut launchpad, staked_amount, &c_clock, &mut ctx);

        // Ensure user is registered correctly
        let amount =iotapad::get_user_participation_amount(&mut launchpad, tx_context::sender(&ctx));
        assert!(amount == 1000, 1);
        transfer::public_transfer(having_amount, tx_context::sender(&ctx));
        iotapad::share_launchpad_object(launchpad);
        clock::destroy_for_testing(c_clock);
    }

    // Test for preventing duplicate user registration
    #[test]
    #[expected_failure(abort_code = iotapad::ERROR_ALREADY_PARTICIPATED)]
    public fun test_register_duplicate_participation() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let staked_amount: Coin<IOTAPAD_TESTS> = mint_for_testing<IOTAPAD_TESTS>(1000, &mut ctx);

        // Create a valid launchpad first
        let c_clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&c_clock);
        let end_time = start_time + 10000;
        let times = vector::singleton(start_time + 5000);
        let percentages = vector::singleton(100);

        let mut launchpad = iotapad::get_create_launchpad<IOTAPAD_TESTS, IOTAPAD_TESTS>(
            1,
            10,
            1000,
            start_time,
            end_time,
            tx_context::sender(&ctx),
            times,
            percentages,
            &mut ctx
        );

        // First registration should succeed
        let having_amount = iotapad::register_launchpad<IOTAPAD_TESTS, IOTAPAD_TESTS>(&mut launchpad, staked_amount, &c_clock, &mut ctx);
        transfer::public_transfer(having_amount, tx_context::sender(&ctx));
        // Try to register the same user again (this should fail)
        let second_staked_amount: Coin<IOTAPAD_TESTS> = mint_for_testing<IOTAPAD_TESTS>(500, &mut ctx);
        let having_amount = iotapad::register_launchpad<IOTAPAD_TESTS, IOTAPAD_TESTS>(&mut launchpad, second_staked_amount, &c_clock, &mut ctx);
        transfer::public_transfer(having_amount, tx_context::sender(&ctx));


        iotapad::share_launchpad_object(launchpad);
        clock::destroy_for_testing(c_clock);
    }

    // Test for successful vesting contract creation after launchpad ends
    #[test]
    #[expected_failure(abort_code = iotapad::ERROR_LAUNCHPAD_STILL_ACTIVE)]
    public fun test_create_vesting_contract_fail_still_active() {
        let mut ctx = tx_context::dummy();  // Initialize a test context

        // Create a valid launchpad first
        let mut c_clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&c_clock);
        let end_time = start_time + 10000;
        let times = vector::singleton(start_time + 5000);
        let percentages = vector::singleton(100);

        let mut launchpad = iotapad::get_create_launchpad<IOTAPAD_TESTS, IOTAPAD_TESTS>(
            1,
            10,
            1000,
            start_time,
            end_time,
            tx_context::sender(&ctx),
            times,
            percentages,
            &mut ctx
        );

        // Simulate launchpad ending
        clock::increment_for_testing(&mut c_clock, 20);

        // Create vesting contract
        let vesting_amount: Coin<IOTAPAD_TESTS> = mint_for_testing<IOTAPAD_TESTS>(1000, &mut ctx);
        iotapad::create_the_vesting<IOTAPAD_TESTS, IOTAPAD_TESTS, IOTAPAD_TESTS>(
            &mut launchpad, 
            clock::timestamp_ms(&c_clock),
            1000,
            &c_clock,
            vesting_amount,
            &mut ctx
        );

        iotapad::share_launchpad_object<IOTAPAD_TESTS, IOTAPAD_TESTS>(launchpad);
        clock::destroy_for_testing(c_clock);
    }
    // Test for successful vesting contract creation after launchpad ends
    #[test]
    #[expected_failure(abort_code = iotapad::ERROR_NOT_ADMIN)]
    public fun test_create_vesting_contract_fail_not_admin() {
        let mut ctx = tx_context::dummy();  // Initialize a test context

        // Create a valid launchpad first
        let mut c_clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&c_clock);
        let end_time = start_time + 10000;
        let times = vector::singleton(start_time + 5000);
        let percentages = vector::singleton(100);

        let mut launchpad = iotapad::get_create_launchpad<IOTAPAD_TESTS, IOTAPAD_TESTS>(
            1,
            10,
            1000,
            start_time,
            end_time,
            tx_context::sender(&ctx),
            times,
            percentages,
            &mut ctx
        );

        // Simulate launchpad ending
        clock::increment_for_testing(&mut c_clock, 20000);

        iotapad::close_launchpad(&mut launchpad, &c_clock, &mut ctx);

        let mut ctx2 = tx_context::new_from_hint(
            tx_context::fresh_object_address(&mut ctx), 676,34, 4344,334334
        );  // Initialize a test context
        // Create vesting contract
        let vesting_amount: Coin<IOTAPAD_TESTS> = mint_for_testing<IOTAPAD_TESTS>(1000, &mut ctx);
        iotapad::create_the_vesting<IOTAPAD_TESTS, IOTAPAD_TESTS, IOTAPAD_TESTS>(
            &mut launchpad, 
            clock::timestamp_ms(&c_clock),
            1000,
            &c_clock,
            vesting_amount,
            &mut ctx2
        );

        iotapad::share_launchpad_object<IOTAPAD_TESTS, IOTAPAD_TESTS>(launchpad);
        clock::destroy_for_testing(c_clock);
    }

    // Test for successful vesting contract creation after launchpad ends
    #[test]
    public fun test_create_vesting_contract_success() {
        let mut ctx = tx_context::dummy();  // Initialize a test context

        // Create a valid launchpad first
        let mut c_clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&c_clock);
        let end_time = start_time + 10000;
        let times = vector::singleton(start_time + 5000);
        let percentages = vector::singleton(100);


        let mut launchpad = iotapad::get_create_launchpad<IOTAPAD_TESTS, IOTAPAD_TESTS>(
            1,
            10,
            1000,
            start_time,
            end_time,
            tx_context::sender(&ctx),
            times,
            percentages,
            &mut ctx
        );

        // Simulate launchpad ending
        clock::increment_for_testing(&mut c_clock, 20000);

        iotapad::close_launchpad(&mut launchpad, &c_clock, &mut ctx);
        // Create vesting contract
        let vesting_amount: Coin<IOTAPAD_TESTS> = mint_for_testing<IOTAPAD_TESTS>(1000, &mut ctx);
        iotapad::create_the_vesting<IOTAPAD_TESTS, IOTAPAD_TESTS, IOTAPAD_TESTS>(
            &mut launchpad, 
            clock::timestamp_ms(&c_clock),
            1000,
            &c_clock,
            vesting_amount,
            &mut ctx
        );

        iotapad::share_launchpad_object<IOTAPAD_TESTS, IOTAPAD_TESTS>(launchpad);
        clock::destroy_for_testing(c_clock);
    }

    #[test]
    public fun test_launchpad_lifecycle() {
        let mut ctx = tx_context::dummy();  // Initialize a test context
        let mut c_clock = clock::create_for_testing(&mut ctx);
        let start_time = clock::timestamp_ms(&c_clock) + 10000;
        let end_time = start_time + 10000;  // End in 10 seconds
        let times = vector::singleton(start_time + 5000);  // Vesting time 5 sec after start
        let percentages = vector::singleton(100);  // 100% vesting

        let mut receipient = tx_context::new_from_hint(
            tx_context::fresh_object_address(&mut ctx), 1, 1, 1, 1
        );

        let mut a_user = tx_context::new_from_hint(
            tx_context::fresh_object_address(&mut ctx), 2, 2, 2, 2
        );

        // Step 1: Create Launchpad
        let mut launchpad = iotapad::get_create_launchpad<IOTAPAD_TESTS, IOTAPAD_TESTS>(
            1,
            10,  // Conversion rate
            1000,  // Total allocation
            start_time,
            end_time,
            tx_context::sender(&receipient),
            times,
            percentages,
            &mut ctx
        );

        //assert!(start_time < clock::timestamp_ms(&c_clock), 9987);
        // Step 2: Register user in the launchpad
        let staked_amount: Coin<IOTAPAD_TESTS> = mint_for_testing<IOTAPAD_TESTS>(1000, &mut a_user);
        let having_amount: Coin<IOTAPAD_TESTS> = iotapad::register_launchpad<IOTAPAD_TESTS, IOTAPAD_TESTS>(
            &mut launchpad, staked_amount, &c_clock, &mut a_user
        );

        // Assert user is registered correctly
        let amount = iotapad::get_user_participation_amount(&mut launchpad, tx_context::sender(&a_user));
        assert!(amount == 1000, 1);

        
        clock::increment_for_testing(&mut c_clock, 12000);  // Move time past end_time
        // Step 3: Participate in Launchpad
        iotapad::participate_in_launchpad(&mut launchpad, having_amount, &c_clock,&mut a_user);

        // Step 4: Close the Launchpad by Admin
        clock::increment_for_testing(&mut c_clock, 30000);  // Move time past end_time
        iotapad::close_launchpad(&mut launchpad, &c_clock, &mut ctx);

        // Step 5: Create Vesting Contract by Admin
        let vesting_amount: Coin<IOTAPAD_TESTS> = mint_for_testing<IOTAPAD_TESTS>(1000, &mut ctx);
        iotapad::create_the_vesting<IOTAPAD_TESTS, IOTAPAD_TESTS, IOTAPAD_TESTS>(
            &mut launchpad,
            clock::timestamp_ms(&c_clock),
            1000,  // Vesting duration
            &c_clock,
            vesting_amount,
            &mut ctx
        );

        // Step 6: Claim Raised Tokens
        iotapad::claim_raised_tokens(&mut launchpad, &c_clock, &mut receipient);

        // Clean up resources
        iotapad::share_launchpad_object(launchpad);
        clock::destroy_for_testing(c_clock);
    }

}
