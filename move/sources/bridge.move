/// Module: bridge
module bridge::bridge {

    //use iota::address;
    use iota::event::emit;
    use std::string::String;
    use iota::iota::IOTA;
    use iota::coin::{Self, Coin};
    use iota::coin_manager::{Self, CoinManagerTreasuryCap, CoinManager};
    use iota::balance::{Self, Balance};
    use bridge::wETH::WETH;

    // Define a struct to hold the IOTA coins
    public struct Bridge has key {
        id: UID,
        locked_funds: Balance<IOTA>,    // IOTA locked 
        weth_to_burn: Balance<WETH>     // returned wETH that must be burnt
    }

    // Admin capability, held by the server
    public struct AdminCap has key {
        id: UID,
    }

    /* Events */
    public struct IOTALocked has copy, drop {
        //sender: address,
        recipient: String,  // Recipient address on the other chain
        amount: u64
    }

    public struct WETHMinted has copy, drop {
        recipient: address,
        amount: u64
    }

    public struct WETHBurned has copy, drop {
        recipient: String,  // Recipient address on the other chain
        amount: u64
    }

    public struct IOTAUnlocked has copy, drop {
        recipient: address,
        amount: u64
    }

    // Initialize Bridge object
    fun init(ctx: &mut TxContext) {

        let bridge = Bridge { 
            id: object::new(ctx), 
            locked_funds: balance::zero<IOTA>(),
            weth_to_burn: balance::zero<WETH>(),
        };

        transfer::share_object(bridge);

        let admin_cap = AdminCap { id: object::new(ctx) };

        transfer::transfer(admin_cap, ctx.sender())
    }

    // Lock IOTA funds 
    // and emit an event that triggers the minting of a corresponding amount of wIOTA  
    // to be sent to the 'recipient' address on the other chain
    public entry fun lockIOTA(
        bridge: &mut Bridge, 
        iota: Coin<IOTA>, 
        recipient: String) {

        assert!(iota.value() > 0, 0);

        emit(IOTALocked { recipient: recipient, amount: iota.value() });

        balance::join(&mut bridge.locked_funds, coin::into_balance(iota));
    }

    // Mint 'amount' of wETH and transfer them to 'recipient'
    public entry fun mintWETH(
        _: &AdminCap,
        cm_treasury_cap: &CoinManagerTreasuryCap<WETH>,
        manager: &mut CoinManager<WETH>, 
        recipient: address, 
        amount: u64, 
        ctx: &mut TxContext) {

        emit(WETHMinted { recipient: recipient, amount: amount });

        let weth = coin_manager::mint(cm_treasury_cap, manager, amount, ctx);
        transfer::public_transfer(weth, recipient);
    }

    // Burn wETH
    // and emit an event that triggers the unlocking of a corresponding amount of ETH  
    // to be sent to the 'recipient' address on the other chain
    // (the wETHs are transferred to 'bridge.weth_to_burn', and will be subsequently burned by a different method)
    public entry fun burnWETH(
        bridge: &mut Bridge, 
        weth: Coin<WETH>, 
        recipient: String) {

        emit(WETHBurned {recipient: recipient, amount: weth.value() });

        //coin_manager::burn(bridge.cm_treasury_cap, manager, weth);
        balance::join(&mut bridge.weth_to_burn, coin::into_balance(weth));
    }

    // Unlock 'amount' of IOTA and transfer them to 'recipient'
    public entry fun unlockIOTA(
        _: &AdminCap,
        bridge: &mut Bridge, 
        recipient: address, 
        amount: u64, 
        ctx: &mut TxContext) {

        emit(IOTAUnlocked {recipient: recipient, amount: amount });

        let unlocked_balance = balance::split(&mut bridge.locked_funds, amount);
        let unlocked_coin = coin::from_balance(unlocked_balance, ctx);

        transfer::public_transfer(unlocked_coin, recipient)
    }

    
    
    #[test]
    public fun fun_lockIOTA_increases_locked_funds() {
        // Test that after the method lockIOTA is called, locked_funds is increased by the exact amout of IOTA locked  
        use iota::test_scenario;
        let admin = @0xCAFE;

        let mut scenario = test_scenario::begin(admin);

        let mut bridge = {
            let ctx = scenario.ctx();
            Bridge { 
                id: object::new(ctx), 
                locked_funds: balance::zero<IOTA>(),
                weth_to_burn: balance::zero<WETH>(),
            }
        };

        {
            let ctx = scenario.ctx();
            let admin_cap = AdminCap { id: object::new(ctx) };
            transfer::transfer(admin_cap, ctx.sender());
        };
        
        let iota = {
            let ctx = scenario.ctx();
            coin::mint_for_testing<IOTA>(10_000_000_000, ctx)
        };
        
        let stri = std::string::utf8(b"test");

        let locked_funds_value_before = balance::value<IOTA>(&bridge.locked_funds);
        let iota_value = coin::value<IOTA>(&iota);

        lockIOTA(&mut bridge, iota, stri);

        let locked_funds_value_after = balance::value<IOTA>(&bridge.locked_funds);

        assert!(locked_funds_value_before + iota_value == locked_funds_value_after, 0);

        transfer::share_object(bridge);

        test_scenario::end(scenario);  
    } 


    #[test]
    #[expected_failure]
    public fun fun_lock_zero_IOTA() {
        // Check that it is not allowed to lock 0 IOTA
        
        use iota::test_scenario;
        let admin = @0xCAFE;

        let mut scenario = test_scenario::begin(admin);

        let mut bridge = {
            let ctx = scenario.ctx();
            Bridge { 
                id: object::new(ctx), 
                locked_funds: balance::zero<IOTA>(),
                weth_to_burn: balance::zero<WETH>(),
            }
        };

        {
            let ctx = scenario.ctx();
            let admin_cap = AdminCap { id: object::new(ctx) };
            transfer::transfer(admin_cap, ctx.sender());
        };
        
        let iota = {
            let ctx = scenario.ctx();
            coin::mint_for_testing<IOTA>(0, ctx)
        };
        
        let stri = std::string::utf8(b"test");

        lockIOTA(&mut bridge, iota, stri);

        transfer::share_object(bridge);

        test_scenario::end(scenario);  
    } 
 
}
