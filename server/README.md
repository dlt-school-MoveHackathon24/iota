# Bridge Server

## Structure

- `src/services/`
  - **`ethereumServices.ts`**: Contains functions for the bridge to call `mintWIOTA` and `unlockETH`. These operations are restricted to the bridge.
  - **`iotaServices.ts`**: Contains functions for the bridge to call `mintWETH` and `unlockIOTA`. These operations are restricted to the bridge.

- `src/utils/`
  - **`iotaIds.ts`**: Functions to dynamically retrieve IOTA object IDs.
  - **`provider.ts`**: Configures the Ethereum provider.

- **`bridgeConfig.ts`**: Contains addresses and keys to run the bridge.

- **`app.ts`**: Main server script. It sets up event listeners and manages bridge operations.

- **Transaction Scripts**: Scripts to interact with the bridge using externally owned accounts:
  - **`lockETH.ts`**: Locks ETH in the Ethereum contract.
  - **`lockIOTA.ts`**: Locks IOTA in the IOTA module.
  - **`burnWETH.ts`**: Burns wrapped ETH through the IOTA module.
  - **`burnWIOTA.ts`**: Burns wrapped IOTA through the Ethereum contract.

## Configuring the Setup

Update the following variables in the `bridgeConfig.ts` configuration file:

- **`bridgeOwnerSK`**: The secret key of the bridge owner. Use the private key of account #0 displayed when starting the Hardhat node.
- **`accountSK`**: The secret key of the externally owned account (EOA) used for running tests. Use the private key of the last account displayed when starting the Hardhat node.
- **`accountPK`**: The public key of the EOA used for running tests. Use the public key of the last account displayed when starting the Hardhat node.
- **`iotaAccountSK`**: The secret key of the IOTA account. To retrieve it:
  1. Run `iota client active-address` to get the currently active address.
  2. Run `iota keytool export --key-identity <your active address>` to export the private key.
  3. Run `iota keytool convert <your private key>` and retrieve the `hexWithoutFlag` value.
  4. Set the `hexWithoutFlag` value in the `iotaAccountSK` variable.
