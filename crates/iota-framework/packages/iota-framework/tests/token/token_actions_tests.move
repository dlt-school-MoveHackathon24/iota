// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This testing block makes sure the protected (restricted) actions behave as
/// intended, that the request is well formed and that APIs are usable.
///
/// It also tests custom actions which can be implemented by policy owner.
module iota::token_actions_tests {
    use iota::token;
    use iota::token_test_utils as test;

    #[test]
    /// Scenario: perform a transfer operation, and confirm that the request
    /// is well-formed.
    fun test_transfer_action() {
        let ctx = &mut test::ctx(@0x0);
        let (policy, cap) = test::get_policy(ctx);

        let token = test::mint(1000, ctx);
        let request = token::transfer(token, @0x2, ctx);

        assert!(request.action() == token::transfer_action(), 0);
        assert!(request.amount() == 1000, 1);
        assert!(request.sender() == @0x0, 2);

        let recipient = request.recipient();

        assert!(recipient.is_some(), 3);
        assert!(recipient.borrow() == &@0x2, 4);
        assert!(request.spent().is_none(), 5);

        cap.confirm_with_policy_cap(request, ctx);
        test::return_policy(policy, cap);
    }

    #[test]
    /// Scenario: spend 1000 tokens, make sure the request is well-formed, and
    /// confirm request in the policy making sure the balance is updated.
    fun test_spend_action() {
        let ctx = &mut test::ctx(@0x0);
        let (mut policy, cap) = test::get_policy(ctx);

        let token = test::mint(1000, ctx);
        let request = token::spend(token, ctx);

        policy.allow(&cap, token::spend_action(), ctx);

        assert!(request.action() == token::spend_action(), 0);
        assert!(request.amount() == 1000, 1);
        assert!(request.sender() == @0x0, 2);
        assert!(request.recipient().is_none(), 3);
        assert!(request.spent().is_some(), 4);
        assert!(request.spent().borrow() == &1000, 5);

        token::confirm_request_mut(&mut policy, request, ctx);

        assert!(token::spent_balance(&policy) == 1000, 6);

        test::return_policy(policy, cap);
    }

    #[test]
    /// Scenario: turn 1000 tokens into Coin, make sure the request is well-formed,
    /// and perform a from_coin action to turn the Coin back into tokens.
    fun test_to_from_coin_action() {
        let ctx = &mut test::ctx(@0x0);
        let (policy, cap) = test::get_policy(ctx);

        let token = test::mint(1000, ctx);
        let (coin, to_request) = token.to_coin(ctx);

        assert!(to_request.action() == token::to_coin_action(), 0);
        assert!(to_request.amount() == 1000, 1);
        assert!(to_request.sender() == @0x0, 2);
        assert!(to_request.recipient().is_none(), 3);
        assert!(to_request.spent().is_none(), 4);

        let (token, from_request) = token::from_coin(coin, ctx);

        assert!(from_request.action() == token::from_coin_action(), 5);
        assert!(from_request.amount() == 1000, 6);
        assert!(from_request.sender() == @0x0, 7);
        assert!(from_request.recipient().is_none(), 8);
        assert!(from_request.spent().is_none(), 9);

        token.keep(ctx);
        cap.confirm_with_policy_cap(to_request, ctx);
        cap.confirm_with_policy_cap(from_request, ctx);
        test::return_policy(policy, cap);
    }

    #[test]
    /// Scenario: create a custom request, allow it in the policy, make sure
    /// that the request matches the expected values, and confirm it in the
    /// policy.
    fun test_custom_action() {
        let ctx = &mut test::ctx(@0x0);
        let (mut policy, cap) = test::get_policy(ctx);
        let custom_action = b"custom".to_string();

        policy.allow(&cap, custom_action, ctx);

        let request = token::new_request(
            custom_action,
            1000,
            option::none(),
            option::none(),
            ctx
        );

        assert!(request.action() == custom_action, 0);
        assert!(request.amount() == 1000, 1);
        assert!(request.sender() == @0x0, 2);
        assert!(request.recipient().is_none(), 3);
        assert!(request.spent().is_none(), 4);

        policy.confirm_request(request, ctx);
        test::return_policy(policy, cap);
    }
}
