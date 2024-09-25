
/// Module: iotaidtest
module iotaidtest::iotaidtest {
    use iota::dynamic_object_field as df;
    use std::string::{Self, String};

    public struct Node has key, store {
        id: UID,
        name: String,
        owner: address,
    }

    public struct Address has key {
        id: UID,
        addr: address,
    }

    // init function called when the contract is deployed
    // create a node with IOTA string and transfer the ownership to the sender
    fun init(ctx: &mut TxContext) {
        let iota = Node {
            id: object::new(ctx),
            name: string::utf8(b"iota"),
            owner: tx_context::sender(ctx),
        };
        transfer::share_object(iota);
    }

    public fun get_node(root: &mut Node, uri: vector<String>): &mut Node {
        let mut current_node = root;
        let mut index = 0;
        while (index < uri.length()) {
                current_node = df::borrow_mut<String, Node>(&mut current_node.id, uri[index]);
            
            index = index + 1;
        };
        current_node
    } 

    //add a subdomain
    public fun add_subdomain(root: &mut Node, uri: vector<String>, child: String, ownership: address, ctx: &mut TxContext) {
        let parent_node = get_node(root, uri);
        assert!(parent_node.owner == tx_context::sender(ctx), 1);
        let subdomain = Node {
            id: object::new(ctx),
            name: child,
            owner: ownership
        };
        df::add<String, Node>(&mut parent_node.id, child, subdomain);
    }

    // remove a subdomain:
    // - check if the sender is the URI owner
    #[allow(lint(self_transfer))]
    public fun remove_subdomain(root: &mut Node, uri: vector<String>, child: String, ctx: &mut TxContext){
        let parent_node = get_node(root, uri);
        assert!(parent_node.owner == tx_context::sender(ctx), 1);
        transfer::transfer(df::remove<String, Node>(&mut parent_node.id, child), tx_context::sender(ctx));
    }




    // get address or DID
    #[allow(lint(self_transfer))]
    public fun get_address(root: &mut Node, uri: vector<String>, ctx: &mut TxContext){
        let node = get_node(root, uri);
        transfer::transfer(Address {
            id: object::new(ctx),
            addr: node.owner
        }, tx_context::sender(ctx));
    }   
}