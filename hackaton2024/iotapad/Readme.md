# IOTAPad README

## Overview
IOTAPad is a decentralized launchpad and vesting platform designed to facilitate token fundraising and distribution. It consists of two main modules:

1. **IOTAPad Launchpad Module**: Manages the fundraising, user participation, and token allocation for a project.
2. **IOTAPad Vesting Module**: Manages the distribution of tokens over time according to a vesting schedule.


# Iotapad Launchpad Smart Contract

The `iotapad::iotapad` module provides a comprehensive framework for managing token launchpads on the IOTA blockchain. This contract allows projects to raise tokens, register participants, allocate tokens to participants, and set up vesting strategies for token distribution.

## Key Features

- **Launchpad Creation**: Create token launchpads with conversion rates, allocation limits, time frames, and vesting schedules.
- **User Registration**: Users can register their participation in a launchpad, staking tokens to secure their allocation in the project.
- **Token Claiming**: Raised tokens can be claimed over time according to predefined time frames.
- **Vesting**: The module integrates with the vesting module, allowing dynamic and linear vesting strategies for token distribution.
- **Admin Controls**: The administrator can manage the launchpad, including closing it, claiming funds, and creating vesting contracts for participants.

## Overview of the Workflow

1. **Launchpad Creation**: The administrator creates a launchpad, specifying the conversion rate between staked and raised tokens, the total allocation, and time frames for claiming the raised amount.
2. **User Registration**: Participants register by staking a specified amount of tokens during the launchpad's active period.
3. **Participation**: Once registered, users can stake the registered amount, and the contract allocates tokens based on the user's share.
4. **Closing the Launchpad**: When the launchpad ends, the administrator closes it, and a vesting contract can be created to distribute the raised tokens gradually to participants.
5. **Claiming Raised Tokens**: The recipient can claim the raised tokens according to the vesting time frames.

## Core Structures

- **Launchpad**: The core structure representing a launchpad. It holds details about the raised tokens, participants, time frames, and vesting information.
- **Participation**: Tracks the amount staked by each participant and the tokens they are eligible to receive.
- **TimeFrame**: Represents the time-based release schedule, allowing the raised tokens to be claimed over time.

## Functions

### 1. **Create a Launchpad**

This function sets up a new launchpad with details such as conversion rate, total allocation, and time frames for claiming the raised tokens.

```move
public entry fun create_launchpad<StakedToken, TokenToPay>(
    conversion_rate_token: u8,          // Conversion rate for tokens
    total_allocation: u64,              // Total allocation of raised tokens
    start_time: u64,                    // Launchpad start time
    end_time: u64,                      // Launchpad end time
    recipient_after_launch: address,    // Recipient of the raised tokens
    times: vector<u64>,                 // Vesting time frames
    percentages: vector<u8>,            // Vesting percentages
    ctx: &mut TxContext
)
```

### 2. **Register for the Launchpad**

Users can register by staking tokens during the active launchpad period. The amount staked determines their share of the raised tokens.

```move
public fun register_launchpad<StakedToken, TokenToPay>(
    launchpad: &mut Launchpad<StakedToken, TokenToPay>,    // Reference to the launchpad
    having_amount: Coin<StakedToken>,                      // Amount the user is staking
    a_clock: &Clock,                                       // Clock to track the launchpad time
    ctx: &mut TxContext                                    // Transaction context
)
```

### 3. **Participate in the Launchpad**

Once registered, users can participate by staking the amount they registered for. The tokens are then allocated based on their share of the total staked amount.

```move
public entry fun participate_in_launchpad<StakedToken, TokenToPay>(
    launchpad: &mut Launchpad<StakedToken, TokenToPay>,    // Reference to the launchpad
    token_to_pay: Coin<TokenToPay>,                        // The amount the user wants to stake
    a_clock: &Clock,                                       // Clock for time validation
    ctx: &mut TxContext                                    // Transaction context
)
```

### 4. **Close the Launchpad**

The administrator can close the launchpad after it ends. Closing the launchpad marks it as inactive, and further participation is disallowed.

```move
public entry fun close_launchpad<StakedToken, TokenToPay>(
    launchpad: &mut Launchpad<StakedToken, TokenToPay>,    // Reference to the launchpad
    clock: &Clock,                                         // Clock for time validation
    ctx: &mut TxContext                                    // Transaction context
)
```

### 5. **Claim Raised Tokens**

After the launchpad is closed, the recipient can claim the raised tokens according to the predefined time frames.

```move
public entry fun claim_raised_tokens<StakedToken, TokenToPay>(
    launchpad: &mut Launchpad<StakedToken, TokenToPay>,    // Reference to the launchpad
    clock: &Clock,                                         // Clock for time validation
    ctx: &mut TxContext                                    // Transaction context
)
```

### 6. **Create a Vesting Contract for Participants**

Once the launchpad is closed, the administrator can create a vesting contract for the participants, allowing them to receive their allocated tokens gradually over time.

```move
public entry fun create_the_vesting<StakedToken, TokenToPay, ClaimToken>(
    launchpad: &mut Launchpad<StakedToken, TokenToPay>,    // Reference to the launchpad
    start_time: u64,                                       // Vesting start time
    duration_seconds: u64,                                 // Vesting duration
    clock: &Clock,                                         // Clock for time validation
    receving_token: Coin<ClaimToken>,                      // The token to be vested
    ctx: &mut TxContext                                    // Transaction context
)
```

### 7. **Release Vested Tokens**

Participants can release their vested tokens based on the vesting schedule.

```move
public entry fun release_vested_coins<Asset>(
    _vester: &mut Vesting<Asset, StrategyType<u64>>, 
    _clock: &Clock,
    ctx: &mut TxContext
)
```

## Error Codes

- `ERROR_LAUNCHPAD_NOT_ACTIVE`: Launchpad is inactive, and participation is not allowed.
- `ERROR_ALREADY_PARTICIPATED`: User has already participated in the launchpad.
- `ERROR_INSUFFICIENT_FUNDS`: User has insufficient funds to participate or claim tokens.
- `ERROR_LAUNCHPAD_CLOSED`: The launchpad is already closed.
- `ERROR_NOT_ADMIN`: Non-administrator users are trying to perform admin actions.
- `ERROR_INVALID_TIME_FRAME_PARAMETERS`: The provided time frames or percentages are invalid.
- `ERROR_LAUNCHPAD_STILL_ACTIVE`: The launchpad is still active and cannot be closed.
- `ERROR_NOT_RECEIPIENT`: The user trying to claim raised tokens is not the designated recipient.

## Testing

Comprehensive tests are provided to ensure the proper functionality of the launchpad, including user registration, participation, closing the launchpad, and vesting tokens.

### Example Test: Launchpad Creation

```move
#[test]
public fun test_create_launchpad() {
    let mut ctx = tx_context::dummy();  // Initialize a test context
    let clock = clock::create_for_testing(&mut ctx);
    let start_time = clock::timestamp_ms(&clock);
    let end_time = start_time + 3600;  // Launchpad ends after 1 hour

    // Create a launchpad
    let launchpad = iotapad::iotapad::create_launchpad<StakedToken, TokenToPay>(
        10,    // Conversion rate
        10000, // Total allocation
        start_time,
        end_time,
        tx_context::sender(&ctx),  // Admin address
        vector[start_time + 1800, start_time + 3600],  // Time frames for claiming
        vector[50, 50],  // Percentages for claiming
        &mut ctx
    );

    assert!(launchpad.is_active == true, 101);
    assert!(launchpad.total_allocation == 10000, 102);
}
```

---

## IOTAPad Vesting Module

# Iotapad Vesting Smart Contract

The `iotapad::vesting` module is a smart contract designed to manage token vesting strategies for projects on the IOTA blockchain. The module provides flexibility for releasing tokens over time through various strategies such as linear distribution, dynamic allocation, and distribution based on token holdings. With the new update, the contract also supports mintable vesting, where tokens are minted at the time of release.

## Key Features

- **Multiple Vesting Strategies**:
  - **Linear Distribution**: Tokens are released in equal portions over the vesting duration.
  - **Dynamic Allocation**: Allows custom token allocation to specific users.
  - **Coin-Based Distribution**: Tokens are distributed based on user holdings of a specific coin.
  
- **Mintable Vesting**: Projects can now vest tokens through minting instead of providing the full token supply upfront.
- **Duration-Based and Time-Frame-Based Vesting**: Support for both linear release over a duration and vesting based on time-frame percentages.
- **Admin Control**: The contract administrator can manage token supply, collect unvested tokens, and execute restricted actions.
- **Error Handling**: Comprehensive error codes ensure proper validation and control over vesting processes.

## Mintable Vesting Overview

The latest update introduces **mintable vesting**, allowing tokens to be minted as they vest. This is particularly useful for projects where the entire token supply isn’t pre-allocated but needs to be gradually released.

### How Mintable Vesting Works:

1. **Set Up a Mintable Treasury**: The administrator adds a mintable treasury (via `TreasuryCap`) to the vesting contract.
2. **Minting Tokens**: When tokens are vested, they are minted and released to users, eliminating the need for upfront token deposits.

### Example Workflow for Mintable Vesting:

```move
let treasury_cap = mint_for_testing_treasury<VESTING_TEST>(1000, &mut ctx);  // Create a treasury cap for minting
vesting::set_mintable_treasury(&mut vesting_contract, treasury_cap, &mut ctx);  // Mark contract as mintable

// Vest tokens and initialize the contract
vesting::initialize_vester_with_coin<VESTING_TEST, u64>(&mut vesting_contract, to_vest, &mut ctx);

// Release tokens when the vesting period comes
vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut ctx);
```

## Available Vesting Strategies

### 1. **Linear Distribution**

Tokens are released in equal parts over the vesting duration. For example, a user allocated 1,000 tokens over one year will receive approximately 83 tokens each month.

- Function: `create_strategy_not_for_coin(1, amount_each)`
- Example: 
  ```move
  let strategy = vesting::create_strategy_not_for_coin(1, 100, &mut ctx);
  ```

### 2. **Dynamic Allocation**

This strategy allows custom token allocations for users. For example, one user can receive 500 tokens while another gets 1,000.

- Function: `create_strategy_not_for_coin(2, 0)` (dynamic)
- Example:
  ```move
  let strategy = vesting::create_strategy_not_for_coin(2, 0, &mut ctx);
  ```

### 3. **Coin-Based Distribution**

Tokens are distributed based on user holdings of a specific coin. This ensures that users holding a predefined amount of `CoinB` are eligible for vesting tokens.

- Function: `create_strategy_for_coin<BaseCoin>(3, amount_each)`
- Example:
  ```move
  let strategy = vesting::create_strategy_for_coin<BaseCoin>(3, 100, &mut ctx);
  ```

## Functions

### 1. **Creating a Vesting Contract**

You can create a vesting contract with linear or time-frame-based strategies.

```move
vesting::create_vester<Asset, Type>(
    start_timestamp,         // Vesting start time
    strategy,                // Vesting strategy (linear, dynamic, or coin-based)
    duration_seconds,        // Duration in seconds (optional)
    time_frames,             // Time frames for release (optional)
    percentages,             // Percentages for release per time frame (optional)
    ctx                      // Transaction context
)
```

### 2. **Minting Tokens for Vesting**

Mintable vesting allows tokens to be minted when released.

```move
vesting::set_mintable_treasury<Asset, Type>(
    vesting_contract,        // Vesting contract to make mintable
    treasury_cap,            // Minting treasury permission
    ctx                      // Transaction context
)
```

### 3. **Adding Supply of Pre-Minted Tokens**

If you are using traditional (non-mintable) vesting, you can pre-load the contract with tokens.

```move
vesting::initialize_vester_with_coin<Asset, Type>(
    vesting_contract,       // Vesting contract to initialize
    to_vest,                // Coin to vest
    ctx                     // Transaction context
)
```

### 4. **Releasing Tokens**

Tokens are released to the user either via linear vesting or based on time frames.

```move
vesting::release_coins<Asset>(
    vesting_contract,       // Vesting contract
    clock,                  // Clock for tracking vesting time
    ctx                     // Transaction context
)
```

### 5. **Minting and Releasing Tokens (Mintable Contracts)**

For mintable contracts, tokens are minted and released during the vesting process.

```move
vesting::release_coins<Asset>(
    vesting_contract,       // Vesting contract (mintable)
    clock,                  // Clock for tracking vesting time
    ctx                     // Transaction context
)
```

### 6. **Collecting Non-Vested Tokens**

After the vesting period ends, the administrator can collect any non-vested tokens.

```move
vesting::collect_not_vested_coins<Asset, Type>(
    vesting_contract,       // Vesting contract
    clock,                  // Clock for vesting period
    ctx                     // Transaction context
)
```

### 7. **Setting Dynamic Allocations for Users**

For dynamic strategies, set custom token allocations for each user.

```move
vesting::set_allocate_amount_per_user<Asset, Type>(
    vesting_contract,       // Vesting contract
    users,                  // Vector of users' addresses
    amounts,                // Vector of amounts for each user
    ctx                     // Transaction context
)
```

## Error Codes

The module uses error codes to prevent invalid operations:

- `ERROR_INVALID_DURATION`: Invalid duration (must be > 0).
- `ERROR_INSUFFICIENT_FUNDS`: Insufficient funds to release or collect tokens.
- `ERROR_TOO_EARLY_RELEASE`: Attempting to release tokens before the vesting period.
- `ERROR_NOT_ADMIN`: Unauthorized user attempting admin-only actions.
- `ERROR_INVALID_STRATEGY`: Invalid vesting strategy.
- `ERROR_INVALID_VESTING_PARAMETERS`: Missing or invalid parameters for vesting.
- `ERROR_VESTING_NOT_ENDED`: Attempting to collect non-vested tokens before the vesting period ends.

## Testing

Comprehensive tests ensure the proper functionality of vesting mechanisms, including token release, dynamic allocation, and minting.

### Example Test: Minting and Vesting

```move
#[test]
public fun test_mint_and_vest_success() {
    let mut ctx = tx_context::dummy();  // Initialize a test context
    let clock = clock::create_for_testing(&mut ctx);
    let start_time = clock::timestamp_ms(&clock);

    // Create a linear vesting strategy
    let strategy = vesting::create_strategy_not_for_coin(1, 100, &mut ctx);

    // Set up a mintable vesting contract
    let treasury_cap = mint_for_testing_treasury<VESTING_TEST>(1000, &mut ctx);
    vesting::set_mintable_treasury(&mut vesting_contract, treasury_cap, &mut ctx);

    // Add tokens to vest (by minting them)
    vesting::initialize_vester_with_coin<VESTING_TEST, u64>(&mut vesting_contract, to_vest, &mut ctx);

    // Release tokens after 30 minutes (50% vested)
    clock::increment_for_testing(&mut clock, 1800);
    vesting::release_coins<VESTING_TEST>(&mut vesting_contract, &clock, &mut ctx);

    // Assert that 500 tokens were vested
    assert!(vesting::get_total_vested(&vesting_contract) == 500, 701);
}
```

---

## Tests
To ensure correct functionality, comprehensive tests for both the **IOTAPad Launchpad** and **Vesting** modules can be found in the respective `iotapad_test` modules. You can use these test functions to verify the correct operation of the modules during development.

To run all tests:
```move
iota move test

INCLUDING DEPENDENCY Iota
INCLUDING DEPENDENCY MoveStdlib
BUILDING iotapad
Running Move unit tests
[ PASS    ] 0x0::vesting::test_time_frame_based_releasable_amount
[ PASS    ] 0x0::vesting_test::test_add_supply
[ PASS    ] 0x0::iotapad_tests::test_create_launchpad_invalid_parameters
[ PASS    ] 0x0::vesting_test::test_collect_not_vested_coins
[ PASS    ] 0x0::iotapad_tests::test_create_launchpad_valid
[ PASS    ] 0x0::vesting_test::test_create_dynamic_strategy
[ PASS    ] 0x0::iotapad_tests::test_create_vesting_contract_fail_not_admin
[ PASS    ] 0x0::vesting_test::test_create_dynamic_vesting_with_timeframes
[ PASS    ] 0x0::iotapad_tests::test_create_vesting_contract_fail_still_active
[ PASS    ] 0x0::vesting_test::test_create_linear_strategy
[ PASS    ] 0x0::iotapad_tests::test_create_vesting_contract_success
[ PASS    ] 0x0::vesting_test::test_create_linear_vesting_with_duration
[ PASS    ] 0x0::iotapad_tests::test_launchpad_lifecycle
[ PASS    ] 0x0::vesting_test::test_duration_based_releasable_amount
[ PASS    ] 0x0::iotapad_tests::test_register_duplicate_participation
[ PASS    ] 0x0::vesting_test::test_invalid_timeframes_and_percentages
[ PASS    ] 0x0::iotapad_tests::test_register_launchpad_success
[ PASS    ] 0x0::vesting_test::test_mint_and_vest_fail_without_treasury
[ PASS    ] 0x0::vesting_test::test_mint_and_vest_success
[ PASS    ] 0x0::vesting_test::test_release_coins_by_coinbase
[ PASS    ] 0x0::vesting_test::test_release_coins_linear_vesting
[ PASS    ] 0x0::vesting_test::test_release_coins_timeframes
Test result: OK. Total tests: 22; passed: 22; failed: 0
```