/// Module that defines a generic type `Guardian<T>` that can only be
/// instantiated with a witness.
module examples::guardian {

    /// The phantom parameter T can only be initialized in the `create_guardian`
    /// function, and the types passed must have the `drop` ability.
    public struct Guardian<phantom T: drop> has key, store {
        id: UID
    }

    /// This function takes an instance of the type T with the `drop` ability as
    /// its first argument. The instance is dropped immediately upon receipt.
    public fun create_guardian<T: drop>(
        _witness: T, ctx: &mut TxContext
    ): Guardian<T> {
        Guardian { id: object::new(ctx) }
    }
}

/// Custom module that makes use of the `guardian`.
module examples::peace_guardian {

    // Import the `guardian` module as a dependency.
    use 0x0::guardian;

    /// This type is intended for single-use only.
    public struct PEACE has drop {}

    /// The module initializer is the optimal way to ensure that
    /// the code is executed only once. When combined with the
    /// Witness pattern, it ensures best practices.
    fun init(ctx: &mut TxContext) {
        transfer::public_transfer(
            guardian::create_guardian(PEACE {}, ctx),
            tx_context::sender(ctx)
        )
    }
}