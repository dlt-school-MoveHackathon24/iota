/* A simple rule for the Voting contract */
methods {
    // Declares the getter for the public state variables as `envfree`
    function lockedFunds() external returns (uint256) envfree;
}

/// @title LockETH increases locked_funds
// Check that after the method lockETH is called, locked_funds is increased by the exact amout of IOTA locked  
// Result: https://prover.certora.com/output/5934372/47efde7963f54dd1b88ad8a038417ba0?anonymousKey=b643b63cb710e03a32fd700e8a8340fd175d63de
rule lockETH_increases_locked_funds(env e) {
    uint256 fundsBefore = lockedFunds();

    bool isInFavor;
    lockETH(e, "addressIOTA");

    assert (
        lockedFunds() == fundsBefore + e.msg.value,
        "locked funds increases after locking"
    );
}

