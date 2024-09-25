/// Module: bridge
module bridge::wETH {

    use iota::coin_manager;

    // One Time Witness
    public struct WETH has drop {}

    fun init(witness: WETH, ctx: &mut TxContext) {
        // Create a `Coin` type and have it managed.
        let (cm_treasury_cap, cm_meta_cap, manager) = coin_manager::create(
            witness,
            18,          // decimals
            b"WETH",
            b"Wrapped Ether",
            b"Wrapped Ether.",
            option::none(),
            ctx
        );

        // Transfer the `CoinManagerTreasuryCap` to the creator of the `Coin`, the server.
        //! To send to the bridge contract
        transfer::public_transfer(cm_treasury_cap, ctx.sender());
        
        // Transfer the `CoinManagerMetadataCap` to the creator of the `Coin`, the server
        transfer::public_transfer(cm_meta_cap, ctx.sender());

        // Publicly share the `CoinManager` object for convenient usage by anyone interested.
        transfer::public_share_object(manager);
    }
}
