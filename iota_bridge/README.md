# IOTA Bridge

The IOTA Bridge is a protocol that facilitates the transfer of assets between IOTA and Ethereum blockchains.

1. [Achievements](#achievements)
1. [Structure](#structure)
1. [How to Run the Bridge](#how-to-run-the-bridge)

## Achievements

- Developed bridge implementations in both Solidity and Move :white_check_mark:
- Created unit tests for Solidity and Move :white_check_mark:
- Formally verified Solidity properties using Certora :white_check_mark:
- Implemented the server :white_check_mark:

## Structure

The content of the project is structured as follows:

- [`solidity/`](solidity/):
  - `contracts/`:
    - **`WIOTA.sol`**: Wrapped IOTA tokens as ERC20.
    - **`Bridge.sol`**: The contract that users interact with to transfer assets to the other side of the chain.
  - `test/`
  - `specs/`: Contains specifications written in CVL (the Certora spec format).

- [`move/`](move/):
  - `sources/`:
    - **`wETH.move`**: Wrapped ETH tokens using the `coin_manager` method provided by IOTA.
    - **`bridge.move`**: The contract that users interact with to transfer assets to the other side of the chain.

- [`server/`](server/): Contains the offchain server implementation of the bridge.


### The Solidity side of the Bridge 

In order to send ```ETH``` from the Ethereum chain to the IOTA chain, the user has to lock the desired amount of Ether by calling the ```lockETH(string calldata addressIOTA)``` method, passing the string of the address on the IOTA chain that will receive the corresponding amount of minted ```wETH```. Then, the server will automatically trigger the minting of ```wETH``` on the IOTA chain and transfer the funds to the specified ```addressIOTA```.

The user that has received minted ```wIOTA``` on the Ethereum chain, and that wants to send them to the IOTA chain to get actual ```IOTA``` coins, has to call the ```burnIOTA(uint256 amount, string calldata addressIOTA)``` method, passing the amount of ```wIOTA``` to burn, and the string of the address on the IOTA chain that will receive the  ```IOTA```.


### The IOTA side of the Bridge 

In order to send ```IOTA``` from the IOTA chain to the Ethereum chain, the user has to lock the desired amount of IOTA by calling the ```lockIOTA(...)``` method, passing as parameters the ```bridge``` object itself, the ```IOTA``` coins, and the ```address``` on the Ethereum chain (as a string) that will receive the corresponding amount of minted ```wIOTA```. Then, the server will automatically trigger the minting of ```wIOTA``` on the Ethereum chain and transfer the funds to the specified ```address```.

The user that has received minted ```wETH``` on the IOTA chain, and that wants to send them to the Ethereum chain to get actual ```ETH``` coins, has to call the ```burnETH(...)``` method, passing  as parameters ```bridge``` object itself, the ```wETH``` coins, and the string of the address on the Ethereum chain that will receive the ```ETH```.

## How to run the Bridge

### Start Local Nodes

To begin, you need to start the Ethereum and IOTA nodes.

**Ethereum Node**: Navigate to the `solidity/` directory and run the following commands:

```
npm install --save-dev hardhat
npx hardhat node
```

**IOTA Node**: Start the IOTA node with the command below:

```
RUST_LOG="off,iota_node=info" iota-test-validator
```

### Deploy Smart Contracts

#### Ethereum Contracts

For deploying Ethereum contracts, we use Hardhat Ignition. Deployment scripts are located in the `solidity/ignition/modules` directory. To deploy the contracts, follow these steps:

**Deploy Wrapped IOTA Contract**: Navigate to the `solidity/` directory and run
```
npx hardhat ignition deploy ignition/modules/WIOTA.ts --network localhost
```

**Deploy Bridge Contract**: Run the following command:

```
npx hardhat ignition deploy ignition/modules/Bridge.ts --network localhost
```

#### IOTA

Before deploying IOTA modules, you need to obtain gas. Use the following command:

```
iota client faucet --url http://127.0.0.1:9123/gas
```

Next, publish the IOTA modules:

```
iota client publish --gas-budget 5000000000 move/sources/bridge.move --json > move/deployed_objects.json
```

This command deploys the bridge and saves the related object IDs to the `move/deployed_objects.json` file, which will be used later by the off-chain server to manage communication between the Ethereum and IOTA blockchains.

### Build and Run the Server
After linking the iota sdk and iota bcs into the `server/` directory, install all the necessary dependencies for your project running `pnpm install`.

Before starting the server, ensure you update the server configuration file by following the instructions outlined [here](server/README.md#configuring-the-setup).

To start the server, run the following command:

```
npx ts-node server/src/app.ts
```

This command initializes listeners for the four main events that can occur across both blockchains:

- `LockETH`
- `BurnWIOTA`
- `IOTALocked`
- `WETHBurned`

### Execute Transactions

Transaction scripts are available in the `server/src/` directory. Each script corresponds to a specific transaction type and can be used to test the bridge functionality:

- **Lock Ethers**: `lockETH.ts`
  - Takes the recipient address and the amount
  - e.g. `npx ts-node lockETH.ts <iota address>`
- **Burn Wrapped IOTA**: `burnWIOTA.ts`
  - Takes the recipient address and the amount
  - e.g. `npx ts-node burnWIOTA.ts <iota address> <amount>`
- **Lock IOTAs**: `lockIOTA.ts`
  - Takes the recipient address and the coin id
  - e.g. `npx ts-node lockIOTA.ts <ethereum address> <iota coin id>`
- **Burn Wrapped Ethers**: `burnWETH.ts`
  - Takes the recipient address and the coin id
  - e.g. `npx ts-node burnWETH.ts <ethereum address> <weth coin id>`

Run these scripts as needed to execute transactions and validate the bridge operations.


## How to run unit tests and specifications

### Run unit tests
To run unit tests for the Move part, run the following command from the `move` directory:

```
iota move test
```

To run unit tests for the Solidity part, run the following command from the `solidity` directory: 

```
npx hardhat test
```

### Run specifications

In order to prove a specification for the Solidity part, run the following command from the ```solidity``` directory:

```certoraRun contracts/Bridge.sol:Bridge --verify Bridge:specs/Bridge.spec --wait_for_results --rule RULE_NAME```

where ```RULE_NAME``` is the name of the rule you want to prove, as written in the ```Bridge.spec``` file.

If you do not have the Certora prover, a link to the results is included as a comment inside ```Bridge.spec``` file for each rule.

