module ctf::SupplyChain {
 
    use iota::event;
    use iota::tx_context;
    use iota::object;
    use iota::transfer;
    use std::vector;
 
    /// Definition of states as constants
    const STATE_OWNED: u8 = 1;
    const STATE_SHARED: u8 = 2;
 
    /// Structure representing the product in the supply chain
    public struct Product has key, store {
        id: UID,               // Unique product identifier
        owner: address,        // Current owner's address
        state: u8,             // Product state (STATE_OWNED or STATE_SHARED)
        producer: address,     // Producer's address
        distributor: address,  // Distributor's address (optional)
        buyer: address,        // Buyer's address (optional)
        sensor_data: u64,      // Recorded temperature
        min_sensor_data: u64,  // Minimum allowed temperature
        max_sensor_data: u64,  // Maximum allowed temperature
        product_validity: bool // Validity of the product based on sensor data
    }

    /// Event for state change
    public struct StateChangedEvent has store, copy, drop {
        product_address: address,
        new_state: u8,
    }

    public fun create_product(
        min_sensor_data: u64,
        max_sensor_data: u64,
        ctx: &mut TxContext,
    ) {
        let producer_address = tx_context::sender(ctx);

        let product = Product {
            id: object::new(ctx),
            owner: producer_address,
            state: STATE_OWNED,
            producer: producer_address,
            distributor: @0x0,  // Initial empty address
            buyer: @0x0,        // Initial empty address
            sensor_data: 0,
            max_sensor_data: max_sensor_data,  // Set the max sensor data
            min_sensor_data: min_sensor_data,  // Set the min sensor data
            product_validity: true,            // Set initial validity to true
        };

        // Emit an event for product creation
        event::emit<StateChangedEvent>(
            StateChangedEvent {
                product_address: object::uid_to_address(&product.id),
                new_state: STATE_OWNED,
            }
        );

        // Storing the product in global storage
        transfer::public_transfer(product, producer_address);
    }


    /// Function to assign the distributor (executed by the producer)
    public fun assign_distributor(
        product: &mut Product,
        distributor: address,
        ctx: &mut TxContext
    ) {
        let producer_address = tx_context::sender(ctx);

        // Verify that the caller is the producer
        assert!(producer_address == product.producer, 1);
        // Verify that the distributor is not already assigned
        assert!(product.distributor == @0x0, 2);
 
        // Assign the distributor
        product.distributor = distributor;
    }
 
    /// Function to assign the buyer (executed by the producer)
    public fun assign_buyer(
        product: &mut Product,
        buyer: address,
        ctx: &mut TxContext
    ) {
        let producer_address = tx_context::sender(ctx);
 
        // Verify that the caller is the producer
        assert!(producer_address == product.producer, 1);
        // Verify that the buyer is not already assigned
        assert!(product.buyer == @0x0, 2);
 
        // Assign the buyer
        product.buyer = buyer;
    }
 
    /// Function to change the product from Owned to Shared
    public fun change_to_shared(
        product: &mut Product,
        ctx: &mut TxContext
    ) {
        let caller_address = tx_context::sender(ctx);
 
        // Verify that the caller is the current owner
        assert!(caller_address == product.owner, 1);

        // Verify that both distributor and buyer are assigned
        assert!(product.distributor != @0x0, 2);
        assert!(product.buyer != @0x0, 3);

        // Update the state
        product.state = STATE_SHARED;
 
        // Emit an event for the state change
        event::emit<StateChangedEvent>(
            StateChangedEvent {
                product_address: object::uid_to_address(&product.id),
                new_state: STATE_SHARED,
            }
        );
    }
 
    /// Function to update sensor data (only in Shared state)
    public fun update_sensor_data(
        product: &mut Product,
        sensor_data: u64,
        ctx: &mut TxContext
    ){
        let participant_address = tx_context::sender(ctx);

        // Verify that the product is in Shared state
        assert!(product.state == STATE_SHARED, 3);

        // Verify that the participant is one of the authorized entities
        assert!(
            participant_address == product.producer ||
            participant_address == product.distributor ||
            participant_address == product.buyer,
            4
        );

        // Update the sensor data
        product.sensor_data = sensor_data;

        // Check if the sensor data is within the valid range
        if (sensor_data < product.min_sensor_data || sensor_data > product.max_sensor_data) {
            // Mark the product as invalid if the data is out of range
            product.product_validity = false;
        }
    }
 
    /// Function to confirm arrival and transfer ownership to the buyer
    public fun confirm_delivery(
        product: &mut Product,
        ctx: &mut TxContext
    ){
        let buyer_address = tx_context::sender(ctx);

        // Verify that the caller is the buyer
        assert!(buyer_address == product.buyer, 1);

        // Update the state
        product.state = STATE_OWNED;
        product.owner = buyer_address;
        
        // Emit an event for the state change
        event::emit<StateChangedEvent>(
            StateChangedEvent {
                product_address: object::uid_to_address(&product.id),
                new_state: STATE_OWNED,
            }
        );
    }
}
