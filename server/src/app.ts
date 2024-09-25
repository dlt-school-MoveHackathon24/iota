import { ethers } from 'ethers';
import { mintWIOTA, unlockETH, getWiotaBalance } from './services/ethereumServices';
import { mintWETH, unlockIOTA } from './services/iotaServices';
import { ethProvider, iotaClient } from './utils/provider';
import { bridgeOwnerSK, bridgeAddress } from './bridgeConfig';
import { getIds } from './utils/iotaIds';
import bridgeJson from '../../solidity/artifacts/contracts/Bridge.sol/Bridge.json';

/* ETHEREUM */

const bridgeWallet = new ethers.Wallet(bridgeOwnerSK, ethProvider);

// Connect to the contract
const bridge = new ethers.Contract(bridgeAddress, bridgeJson.abi, bridgeWallet);

async function listenForLockedETHEvent() {

    // Define the event listener for 'lockedETH'
    bridge.on('LockETH', (from: string, addressIOTA: string, amount: ethers.BigNumberish, event: ethers.ContractEvent) => {
        //console.log(`LockETH event detected from: ${from}, amount: ${ethers.formatEther(amount.toString())}, to: ${addressIOTA}`);
        console.log(`LockETH event detected from: ${from}, amount: ${amount.toString()}, to: ${addressIOTA}`);

        // Mint wrapped ethers
        mintWETH(addressIOTA, parseInt(amount.toString()));

        console.log(`${amount} wrapped ethers minted.`);
    });

    console.log(`Listening for lockedETH events from contract: ${bridgeAddress}`);
}

async function listenForBurnWIOTAEvent() {

    // Define the event listener for 'BurnIOTA'
    bridge.on('BurnWIOTA', (from: string, addressIOTA: string, amount: ethers.BigNumberish, event: ethers.ContractEvent) => {
        console.log(`BurnWIOTA event detected from: ${from}, amount: ${amount.toString()}, IOTA address: ${addressIOTA}`);

        unlockIOTA(addressIOTA, parseInt(amount.toString()));

        console.log(`${amount} IOTAs unlocked.`);
    });

    console.log(`Listening for BurnWIOTA events from contract: ${bridgeAddress}`);
}

/* IOTA */

const bridgePackageId = getIds().bridgePackageId;

async function listenForLockedIOTAEvent() {

    let subscriptionId = await iotaClient.subscribeEvent({
        filter: { "MoveEventType": `${bridgePackageId}::bridge::IOTALocked` },
        onMessage: async (event) => {

            console.log(`IOTALocked event detected`);

            if (typeof event.parsedJson === 'object' && event.parsedJson !== null) {
                const { amount, recipient } = event.parsedJson as { amount?: number ; recipient?: string };

                console.log('Amount:', amount);
                console.log('Recipient:', recipient);

                // Call Ethereum mintWIOTA function
                await mintWIOTA(recipient, amount);

                // Display balance
                const balance = await getWiotaBalance(recipient);
                console.log('new WIOTA balance:', balance);

            } else {
                console.error('event.parsedJson is not an object');
            }
        }
    });

    console.log(`Listening for IOTALocked events from contract: ${bridgePackageId}`);
}

async function listenForBurnedWETHEvent() {

    let subscriptionId = await iotaClient.subscribeEvent({
        filter: { "MoveEventType":`${bridgePackageId}::bridge::WETHBurned` },
        onMessage: async (event) => {

            console.log(`WETHBurned event detected, unlocking ethers...`);

            if (typeof event.parsedJson === 'object' && event.parsedJson !== null) {
                const { amount, recipient } = event.parsedJson as { amount?: number ; recipient?: string };

                console.log('Amount:', amount);
                console.log('Recipient:', recipient);

                await unlockETH(recipient, amount);

                console.log(`${amount} ethers unlocked.`);

            } else {
                console.error('event.parsedJson is not an object');
            }

        },
    });

    console.log(`Listening for WETHBurned events from contract: ${bridgePackageId}`);
}

listenForLockedETHEvent().catch((error) => {
    console.error('Error setting up event listener:', error);
});

listenForBurnWIOTAEvent().catch((error) => {
    console.error('Error setting up event listener:', error);
});

listenForLockedIOTAEvent().catch((error) => {
    console.error('Error setting up event listener:', error);
});

listenForBurnedWETHEvent().catch((error) => {
    console.error('Error setting up event listener:', error);
});