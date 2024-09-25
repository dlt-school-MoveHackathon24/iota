import { ethers } from 'ethers';
import { ethProvider } from './utils/provider';
import bridgeJson from '../../solidity/artifacts/contracts/Bridge.sol/Bridge.json';
import { accountSK, bridgeAddress } from './bridgeConfig';

const wallet = new ethers.Wallet(accountSK, ethProvider);

// Connect to the contract
const bridge = new ethers.Contract(bridgeAddress, bridgeJson.abi, wallet);

async function lock(recipientAddress, amount) {
    try {
        // Send a transaction to lock 1 ETH
        const tx = await bridge.lockETH(recipientAddress, { value: amount });
        console.log(`Transaction sent: ${tx.hash}`);

        // Wait for the transaction to be mined
        const receipt = await tx.wait();

        // Check if the event 'lockedETH' was emitted
        const event = receipt.events?.find((event) => event.event === "LockETH");

        if (event) {
            console.log(`LockETH event emitted: from ${event.args?.from}, amount ${ethers.formatEther(event.args?.amount)}`);
        } else {
            console.log("LockETH event not found in transaction receipt.");
        }

        console.log(`Transaction confirmed, block number: ${receipt.blockNumber}`);

    } catch (error) {
        console.error('Error locking ETH:', (error as Error).message);
    }
}

// Call the lock function
lock(process.argv[2], process.argv[3]);