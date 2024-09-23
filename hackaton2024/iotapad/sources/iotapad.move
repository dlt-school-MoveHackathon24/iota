module iotapad::iotapad {
    use iota::coin::{Self, Coin};
    use iota::balance::{Self, Balance};
    use iotapad::vesting::{Vesting, StrategyType, release_coins, create_strategy_not_for_coin, create_and_initialize_vester, set_amount_by_vec};
    use iota::dynamic_field as df;
    use iota::clock::{Self, Clock};
    use iota::table::{Self, Table};

    // Structure representing the amount of tokens vested to a user
    public struct AmountTo has store {
        amount: u64,         // Amount of tokens vested to the user
    }

    public struct TimeFrame has copy, drop, store {
        time: u64,       // Time in seconds or appropriate units
        percentage: u8,  // Percentage of the total to be released at this time
    }


    // Structure to track user participation in the launchpad
    public struct Participation has store {
        amount: u64,       // Amount of tokens held by the participant
        amount_to_send: u64 // Amount of token to send to user
    }

    // Core structure representing the launchpad contract
    public struct Launchpad<phantom StakedToken, phantom TokenToPay> has key, store {        
        id: UID,                              
        raised_token: Balance<TokenToPay>,    // Token being raised
        conversion_rate_token_from: u64,      // Numerator of the conversion rate
        conversion_rate_token_to: u64,        // Denominator of the conversion rate
        total_staked: u64,                    // Total staked tokens
        start_time: u64,                      // Launchpad start time
        end_time: u64,                        // Launchpad end time
        total_allocation: u64,                // Total token allocation for the participants
        administrator: address,               // Admin address
        recipient_after_launch: address,      // Recipient of the raised funds after launch
        time_frame_claim_raised_amount: vector<TimeFrame>,  // Time frames for claiming raised amounts
        participants: Table<address, Participation>,  // List of participants
        participants_addresses: vector<address>,  // List of participants' address
        is_active: bool,                      // Flag to check if the launchpad is active
        vesting_contract: Option<ID>, // Vesting contract created after launch
    }

    // <<<<<< Error codes >>>>>>
    const ERROR_LAUNCHPAD_NOT_ACTIVE: u64 = 1;          // Error code when trying to participate in an inactive Launchpad
    const ERROR_ALREADY_PARTICIPATED: u64 = 2;          // Error code when a user attempts to participate more than once
    const ERROR_INSUFFICIENT_FUNDS: u64 = 3;            // Error code when a user tries to stake without sufficient tokens
    const ERROR_LAUNCHPAD_CLOSED: u64 = 4;              // Error code when trying to close an already closed Launchpad
    const ERROR_NOT_ADMIN: u64 = 5;                     // Error code when non-admin tries to manage the Launchpad
    const ERROR_INVALID_TIME_FRAME_PARAMETERS: u64 = 7; // Error code for invalid time frame parameters (e.g., empty frames or zero percentages)
    const ERROR_LAUNCHPAD_STILL_ACTIVE: u64 = 9;        // Error code when attempting to create a vesting contract while the launchpad is still active
    const PARTICIPANT_NOT_REGISTERED: u64 = 10;         // Error code when a user tries to participate but is not registered in the launchpad
    const ERROR_REGISTRATION_PASSED: u64 = 11;          // Error code when trying to register after the registration period has passed
    const ERROR_NOT_RECEIPIENT: u64 = 12;               // Error code when a non-designated recipient tries to claim raised tokens

    
    // <<<<<< Launchpad functions >>>>>>

    // Function to create a new Launchpad
    #[allow(lint(share_owned))]
    public entry fun create_launchpad<StakedToken, TokenToPay>(
        conversion_rate_token_from: u64,      // Numerator of the conversion rate
        conversion_rate_token_to: u64,        // Denominator of the conversion rate
        total_allocation: u64,               // Total allocation of raised token
        start_time: u64,                     // Start time of the Launchpad
        end_time: u64,                       // End time of the Launchpad
        recipient_after_launch: address,     // Recipient of the raised tokens
        times: vector<u64>,                 // Vesting time frames
        percentages: vector<u8>,            // Vesting percentages
        ctx: &mut TxContext
    ) {
        transfer::share_object(i_get_launchpad<StakedToken, TokenToPay>(
            conversion_rate_token_from,
            conversion_rate_token_to,
            total_allocation,
            start_time,
            end_time,
            recipient_after_launch,
            times,
            percentages,
            ctx
        ));
    }

    // Register user participation in the launchpad
    public fun register_launchpad<StakedToken, TokenToPay>(
        launchpad: &mut Launchpad<StakedToken, TokenToPay>,    // Reference to the launchpad
        having_amount: Coin<StakedToken>,           // The amount the user is staking
        a_clock: &Clock,
        ctx: &mut TxContext                                    // Transaction context
    ): Coin<StakedToken> {
        let sender = tx_context::sender(ctx);

        // Ensure the launchpad is active and within the correct time frame
        assert!(launchpad.is_active, ERROR_LAUNCHPAD_NOT_ACTIVE);
        let current_time = clock::timestamp_ms(a_clock);
        assert!(
             launchpad.start_time >= current_time,
             ERROR_REGISTRATION_PASSED
        );

        // Ensure the user has not already participated
        assert!(
            !table::contains(&launchpad.participants, sender),
            ERROR_ALREADY_PARTICIPATED
        );

        let staked_value = coin::value(&having_amount);
        assert!(staked_value > 0, ERROR_INSUFFICIENT_FUNDS);

        // Add the participant to the participants table
        let participation = Participation {
            amount: staked_value,   // Set the staked token amount here
            amount_to_send: 0
        };
        table::add(&mut launchpad.participants, sender, participation);

        vector::push_back(&mut launchpad.participants_addresses, sender);

        // Update the total staked amount in the launchpad
        launchpad.total_staked = launchpad.total_staked + staked_value;
        having_amount
    }


    // User participation in the launchpad
    public entry fun participate_in_launchpad<StakedToken, TokenToPay>(
        launchpad: &mut Launchpad<StakedToken, TokenToPay>,
        token_to_pay: Coin<TokenToPay>,  // The amount the user wants to stake
        a_clock: &Clock,
        ctx: &mut TxContext
    ) {
        let sender = tx_context::sender(ctx);
        let current_time = clock::timestamp_ms(a_clock);

        // Ensure the launchpad is active
        assert!(launchpad.is_active && launchpad.start_time <= current_time && launchpad.end_time >= current_time, ERROR_LAUNCHPAD_NOT_ACTIVE);

        // Ensure the user is registered (check if the user exists in the participants table)
        assert!(table::contains(&launchpad.participants, sender), PARTICIPANT_NOT_REGISTERED);

        // Borrow the participant from the table
        let participant = table::borrow_mut(&mut launchpad.participants, sender);

        // Ensure the user has not already participated fully (i.e., amount_to_send is not already set)
        assert!(participant.amount_to_send == 0, ERROR_ALREADY_PARTICIPATED);

        // Calculate the maximum amount the user is allowed to participate with
        let user_registered_amount = participant.amount;  // The amount the user registered with

        // Receive the tokens from the user (TokenToPay)
        let received_value = coin::value(&token_to_pay);
        assert!(received_value > 0 && received_value == user_registered_amount, ERROR_INSUFFICIENT_FUNDS);

        /*
        // If the user sends more than they registered for, refund the extra
        let mut staked_value_to_use = received_value;
        if (received_value > user_registered_amount) {
            let extra_amount = received_value - user_registered_amount;

            // Refund the extra tokens
            let extra_tokens = coin::split(&mut token_to_pay, extra_amount, ctx);
            transfer::public_transfer(extra_tokens, sender);

            staked_value_to_use = user_registered_amount;  // Only the registered amount will be used for calculation
        };*/

        // Calculate the user's share of the launchpad tokens using the conversion rate
        let staked_value_in_target_tokens = (received_value * launchpad.conversion_rate_token_to) / launchpad.conversion_rate_token_from;

        // Calculate the user's share based on their staked amount and the launchpad total allocation
        let user_share = (staked_value_in_target_tokens * launchpad.total_allocation) / launchpad.total_staked;

        // Register the participant's `amount_to_send`
        participant.amount_to_send = user_share;

        // Update launchpad's total staked tokens (with the amount actually used)
        launchpad.total_staked = launchpad.total_staked + received_value;

        // Add the used staked tokens to the raised token balance in the launchpad
        balance::join(&mut launchpad.raised_token, coin::into_balance(token_to_pay));
    }


    // Close the launchpad
    public entry fun close_launchpad<StakedToken, TokenToPay>(
        launchpad: &mut Launchpad<StakedToken, TokenToPay>,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let admin = tx_context::sender(ctx);
        // Get the current timestamp
        let current_time = clock::timestamp_ms(clock);
        // Ensure only the admin can close the launchpad
        assert!(admin == launchpad.administrator, ERROR_NOT_ADMIN);

        // Ensure the launchpad is still active
        assert!(launchpad.is_active && launchpad.end_time <= current_time, ERROR_LAUNCHPAD_CLOSED);

        // Set the launchpad as inactive
        launchpad.is_active = false;

        df::add(&mut launchpad.id, b"alredy_claimed_raised", AmountTo {
                amount: 0
        });

        // The raised tokens will be gradually claimed by the recipient according to the time frames,
        // so we do not transfer any tokens at this point.
    }

    // Claim raised tokens according to the time frame
    public entry fun claim_raised_tokens<StakedToken, TokenToPay>(
        launchpad: &mut Launchpad<StakedToken, TokenToPay>,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let recipient = tx_context::sender(ctx);
        
        // Ensure the sender is the recipient of the raised tokens
        assert!(recipient == launchpad.recipient_after_launch, ERROR_NOT_RECEIPIENT);

        let amountTo: &mut AmountTo = df::borrow_mut(&mut launchpad.id, b"alredy_claimed_raised");
        let claimed_amount = amountTo.amount;
        
        // Get the current timestamp
        let current_time = clock::timestamp_ms(clock);

        // Total claimable amount based on the time frames
        let mut total_claimable: u64 = 0;

        // Calculate the total amount the recipient can claim based on time frames
        let time_frames = &launchpad.time_frame_claim_raised_amount;
        let raised_amount = balance::value(&launchpad.raised_token);
        let num_frames = vector::length(time_frames);
        let mut i = 0;

        while (i < num_frames) {
            let frame = vector::borrow(time_frames, i);

            // Check if the current time has passed the frame's time, allowing claim
            if (current_time >= frame.time) {
                // Calculate the percentage of the raised tokens that can be claimed in this frame
                let claimable_in_frame = (frame.percentage as u64 * raised_amount) / 100;
                total_claimable = total_claimable + claimable_in_frame;
            } else {
                // Stop checking further frames since they are in the future
                break
            };
            i = i + 1;
        };

        // Ensure there are claimable funds
        assert!(total_claimable > claimed_amount && balance::value(&launchpad.raised_token) >= total_claimable - claimed_amount, ERROR_INSUFFICIENT_FUNDS);

        // Split the balance for the claimable amount
        let mut claimable_tokens_balance = balance::split(&mut launchpad.raised_token, (total_claimable - claimed_amount));
        let claimable_tokens = coin::take(&mut claimable_tokens_balance, (total_claimable - claimed_amount), ctx);

        amountTo.amount = total_claimable;
        // Transfer the claimable tokens to the recipient
        transfer::public_transfer(claimable_tokens, recipient);
        balance::destroy_zero(claimable_tokens_balance);  // Destroy any zero balances
    }


    // Create a vesting contract for the participants
    public entry fun create_the_vesting<StakedToken, TokenToPay, ClaimToken>(
        launchpad: &mut Launchpad<StakedToken, TokenToPay>,
        start_time: u64,
        duration_seconds: u64,
        clock: &Clock,
        receving_token: Coin<ClaimToken>,  // The token to be vested
        ctx: &mut TxContext
    ) {
        
        let current_time = clock::timestamp_ms(clock);
        let admin = tx_context::sender(ctx);

        // Ensure only the admin can create the vesting contract
        assert!(admin == launchpad.administrator, ERROR_NOT_ADMIN);

        // Ensure the launchpad is not active anymore
        assert!(!launchpad.is_active && launchpad.end_time <= current_time, ERROR_LAUNCHPAD_STILL_ACTIVE);

        // Create vesting strategy (Linear distribution)
        let mut strategy = create_strategy_not_for_coin(
            2,  // Strategy type: Dynamic key-value distribution (not fixed per user)
            0,  // This is for dynamic strategy, so amount will be added per user
            ctx
        );

        // Prepare the users and amounts vectors to pass to `set_amount_by_vec`
        let mut users: vector<address> = vector::empty();   // Initialize empty vector of users (addresses)
        let mut amounts: vector<u64> = vector::empty();     // Initialize empty vector of amounts

        let num_participants = vector::length(&launchpad.participants_addresses);
        let mut i = 0;

        while (i < num_participants) {
            let user_address = *vector::borrow(&launchpad.participants_addresses, i);

            // Borrow the corresponding participant entry from the participants table
            let participant = table::borrow(&launchpad.participants, user_address);

            let amount = participant.amount_to_send;  // Amount to be vested for the user

            // Add user addresses and their corresponding amounts to the vectors
            vector::push_back(&mut users, user_address);
            vector::push_back(&mut amounts, amount);

            i = i + 1;  // Move to the next participant
        };

        // Pass the users and amounts vectors to the `set_amount_by_vec` function
        set_amount_by_vec(&mut strategy, users, amounts);

        // Receive the tokens to be vested
        //let coin_to_vest = transfer::public_receive<Coin<ClaimToken>>(&mut launchpad.id, receving_token);

        // Initialize the vesting contract with the vesting amount
        let vesting_id = create_and_initialize_vester<ClaimToken, u64>(
            strategy,
            start_time,
            option::some(duration_seconds),
            option::none(),   // No specific time frames here, linear vesting
            option::none(),
            receving_token,
            ctx
        );

        // Add the vesting contract ID to the launchpad
        launchpad.vesting_contract = option::some(vesting_id);
    }

    public entry fun release_vested_coins<Asset>(
        _vester: &mut Vesting<Asset, StrategyType<u64>>, 
        _clock: &Clock,
        ctx: &mut TxContext
    ) {
        release_coins(_vester, _clock, ctx);
    }

    public fun borrow_participants<StakedToken, TokenToPay>(
        launchpad: &mut Launchpad<StakedToken, TokenToPay>,
    ): &Table<address, Participation> {
        &launchpad.participants 
    }

    public fun get_user_participation_amount<StakedToken, TokenToPay>(
        launchpad: &mut Launchpad<StakedToken, TokenToPay>,
        user: address
    ) : u64 {
        if (table::contains(&launchpad.participants, user)) {
            return table::borrow(&launchpad.participants, user).amount
        }; 
        0
    }

    // <<<< Internal functions >>>>
    // Function to create a new Launchpad
    fun i_get_launchpad<StakedToken, TokenToPay>(
        conversion_rate_token_from: u64,      // Numerator of the conversion rate
        conversion_rate_token_to: u64,        // Denominator of the conversion rate
        total_allocation: u64,                // Total allocation of raised token
        start_time: u64,                      // Start time of the Launchpad
        end_time: u64,                        // End time of the Launchpad
        recipient_after_launch: address,      // Recipient of the raised tokens
        times: vector<u64>,                  // Vesting time frames
        percentages: vector<u8>,             // Vesting percentages
        ctx: &mut TxContext
    ) : Launchpad<StakedToken, TokenToPay> {
        let admin = tx_context::sender(ctx);
        // Ensure that times and percentages vectors are of equal length
        assert!(
            vector::length(&times) == vector::length(&percentages),
            ERROR_INVALID_TIME_FRAME_PARAMETERS
        );
        
        let mut time_frame_claim_raised_amount: vector<TimeFrame> = vector::empty();

        let len = vector::length(&times);
        let mut i = 0;
        while (i < len) {
            let time = *vector::borrow(&times, i);
            let percentage = *vector::borrow(&percentages, i);
            // Build the time frame and push to frames vector
            vector::push_back(
                &mut time_frame_claim_raised_amount,
                TimeFrame { time, percentage }
            );
            i = i + 1;
        };
        
        // Create a new Launchpad instance
        let launchpad = Launchpad<StakedToken, TokenToPay> {
            id: object::new(ctx),
            raised_token: balance::zero<TokenToPay>(),
            conversion_rate_token_from,        // Using numerator for conversion rate
            conversion_rate_token_to,          // Using denominator for conversion rate
            total_staked: 0,
            start_time,
            end_time,
            total_allocation,
            administrator: admin,
            recipient_after_launch,
            time_frame_claim_raised_amount,
            participants: table::new<address, Participation>(ctx),
            participants_addresses: vector::empty(),
            is_active: true,   // Launchpad starts as active
            vesting_contract: option::none(), // Vesting contract will be created later
        };

        // Share the new Launchpad object
        return launchpad
    }


    #[test_only]
    #[allow(lint(custom_state_change))]
    public fun share_launchpad_object<StakedToken, TokenToPay>(
        launchpad: Launchpad<StakedToken, TokenToPay>
    )
    {
       transfer::share_object(launchpad);
    }

    #[test_only]
    public fun get_create_launchpad<StakedToken, TokenToPay>(
        conversion_rate_token_from: u64,      // Numerator of the conversion rate
        conversion_rate_token_to: u64,        // Denominator of the conversion rate
        total_allocation: u64,               // Total allocation of raised token
        start_time: u64,                     // Start time of the Launchpad
        end_time: u64,                       // End time of the Launchpad
        recipient_after_launch: address,     // Recipient of the raised tokens
        times: vector<u64>,                  // Vesting time frames
        percentages: vector<u8>,             // Vesting percentages
        ctx: &mut TxContext
    ): Launchpad<StakedToken, TokenToPay> {

        return (i_get_launchpad(
            conversion_rate_token_from,
            conversion_rate_token_to,
            total_allocation,
            start_time,
            end_time,
            recipient_after_launch,
            times,
            percentages,
            ctx
        ))    
    }
}