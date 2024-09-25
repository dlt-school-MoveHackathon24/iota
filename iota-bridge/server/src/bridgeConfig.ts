import deployedAddresses from '../../solidity/ignition/deployments/chain-31337/deployed_addresses.json';
import { fromHEX } from '@iota/bcs';
import { Ed25519Keypair } from '@iota/iota-sdk/keypairs/ed25519';

// Account #0, the one used by hardhat to deploy the contracts and the owner of the Bridge
const bridgeOwnerSK = '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80'; 

// EOA account
const accountSK = '0xdf57089febbacf7ba0bc227dafbffa9fc08a93fdc68e1e42411a14efcf23656e';
const accountPK = '0x8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199';

// IOTA account
const iotaAccountSK = '0xfee44b31a183f4633a0efcaa2f36f7500b2cd841d2729f077a0c42915d351230';

const iotaKeypair = Ed25519Keypair.fromSecretKey(
    fromHEX(iotaAccountSK)
);

const wiotaAddress = deployedAddresses['BridgeModule#WIOTA'];
const bridgeAddress = deployedAddresses['BridgeModule#Bridge'];

export { bridgeOwnerSK, accountSK, accountPK, iotaKeypair, wiotaAddress, bridgeAddress }