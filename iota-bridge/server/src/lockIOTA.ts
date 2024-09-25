import { iotaClient } from './utils/provider';
import { TransactionBlock } from '@iota/iota-sdk/transactions';
import { iotaKeypair } from './bridgeConfig';
import { getIds } from './utils/iotaIds';

const bridgeId = getIds().bridgeId;
const bridgePackageId = getIds().bridgePackageId;

async function lockIOTA(recipientAddress, coinId) {

    const txb = new TransactionBlock();

    txb.moveCall({
        target: `${bridgePackageId}::bridge::lockIOTA`,
        arguments: [
            txb.object(bridgeId),
            txb.object(coinId),
            txb.pure.string(recipientAddress) // test address
        ]
    });

    txb.setGasBudget(10000000);

    const result = await iotaClient.signAndExecuteTransactionBlock({
        signer: iotaKeypair,
        transactionBlock: txb,
        options: {
            showInput: true,
            showEffects: true
        }
    });

    console.log(result);

    await iotaClient.waitForTransactionBlock({ digest: result.digest });
}

lockIOTA(process.argv[2], process.argv[3]);