import { ethers } from 'ethers';
import { ethProvider } from './utils/provider';
import bridgeJson from '../../solidity/artifacts/contracts/Bridge.sol/Bridge.json';
import WIOTAJson from '../../solidity/artifacts/contracts/WIOTA.sol/WIOTA.json';
import { accountSK, accountPK, bridgeAddress, wiotaAddress } from './bridgeConfig';

const wallet = new ethers.Wallet(accountSK, ethProvider);

// Connect to the bridge
const bridge = new ethers.Contract(bridgeAddress, bridgeJson.abi, wallet);

// Connect to the wiota
const wiota = new ethers.Contract(wiotaAddress, WIOTAJson.abi, wallet);

async function burnWIOTA(recipientAddress, amount) {
    try {

        let nonce = await ethProvider.getTransactionCount(accountPK);

        // Amount of tokens the bridge is already allowed to burn
        let bridgeAllowance = await wiota.allowance(accountPK, bridgeAddress);

        if (bridgeAllowance < amount) {
            // Allow wiota to be burned by the bridge
            let tx = await wiota.approve(bridgeAddress, amount - parseInt(bridgeAllowance), {nonce: nonce});
            console.log(`Transaction sent: ${tx.hash}`);

            // Wait for the transaction to be mined
            const receipt = await tx.wait();
            console.log(`Transaction confirmed, block number: ${receipt.blockNumber}`);

            nonce = nonce + 1;
        }


        // Send a transaction to burn 1 WIOTA
        const tx = await bridge.burnWIOTA(amount, recipientAddress, {nonce: nonce});
        console.log(`Transaction sent: ${tx.hash}`);

        // Wait for the transaction to be mined
        const receipt = await tx.wait();

        // Check if the event 'lockedETH' was emitted
        const event = receipt.events?.find((event) => event.event === "BurnWIOTA");

        if (event) {
            console.log(`BurnWIOTA event emitted: from ${event.args?.from}, amount ${ethers.formatEther(event.args?.amount)}`);
        } else {
            console.log("BurnWIOTA event not found in transaction receipt.");
        }

        console.log(`Transaction confirmed, block number: ${receipt.blockNumber}`);

    } catch (error) {
        console.error('Error burning WIOTA:', (error as Error).message);
    }
}

// Call the lock function
burnWIOTA(process.argv[2], process.argv[3]);