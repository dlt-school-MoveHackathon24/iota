import { iotaClient } from '../utils/provider';
import { fromHEX } from '@iota/bcs';
import { Ed25519Keypair } from '@iota/iota-sdk/keypairs/ed25519';
import { TransactionBlock } from '@iota/iota-sdk/transactions';

import { getIds } from '../utils/iotaIds';

let { bridgeId, adminCapId, bridgePackageId, coinManagerId, coinManagerTreasuryCapId } = getIds();

const keypair = Ed25519Keypair.fromSecretKey(
    fromHEX("0xfee44b31a183f4633a0efcaa2f36f7500b2cd841d2729f077a0c42915d351230")
);

/**
 * Calls the mintWETH function on the smart contract to mint WETH tokens.
 * @param {string} recipient - The recipient address.
 * @param {number} amount - The amount of WETH tokens to mint.
 */
async function mintWETH(recipient: string, amount: number) {
    try {
        // Ensure the amount is greater than zero
        if (amount <= 0) {
            throw new Error("Amount must be greater than zero");
        }

        const txb = new TransactionBlock();

        txb.moveCall({
            target: `${bridgePackageId}::bridge::mintWETH`,
            arguments: [
                txb.object(adminCapId),
                txb.object(coinManagerTreasuryCapId), 
                txb.object(coinManagerId),
                txb.pure.address(recipient), 
                txb.pure.u64(amount)],
        });

        txb.setGasBudget(10000000);

        const result = await iotaClient.signAndExecuteTransactionBlock({ 
            signer: keypair, 
            transactionBlock: txb,
            options: {
                showInput: true,
                showEffects: true
            } });

        console.log(result);

        await iotaClient.waitForTransactionBlock({ digest: result.digest });

    } catch (error) {
        console.error(`Error minting WETH tokens: ${(error as Error).message}`);
    }
}

/**
 * Calls the unlockIOTA function on the smart contract.
 * @param {string} recipient - The address that will receive the IOTA.
 * @param {number} amount - The amount of IOTA to unlock.
 */
async function unlockIOTA(recipient: string, amount: number) {
    try {
        // Ensure the amount is greater than zero
        if (amount <= 0) {
            throw new Error("Amount must be greater than zero");
        }

        const txb = new TransactionBlock();

        console.log(txb.pure.address(recipient));

        txb.moveCall({
            target: `${bridgePackageId}::bridge::unlockIOTA`,
            arguments: [
                txb.object(adminCapId),
                txb.object(bridgeId), 
                txb.pure.address(recipient), 
                txb.pure.u64(amount)],
        });

        txb.setGasBudget(10000000);

        const result = await iotaClient.signAndExecuteTransactionBlock({ 
            signer: keypair, 
            transactionBlock: txb,
            options: {
                showInput: true,
                showEffects: true
            } });

        console.log(result);

        await iotaClient.waitForTransactionBlock({ digest: result.digest });

    } catch (error) {
        console.error(`Error burning IOTA tokens: ${(error as Error).message}`);
    }
}

export { mintWETH, unlockIOTA };