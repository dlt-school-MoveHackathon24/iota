import { ethers } from 'ethers';
import { ethProvider } from '../utils/provider';
import { bridgeOwnerSK, wiotaAddress, bridgeAddress } from '../bridgeConfig';
import bridgeJson from '../../../solidity/artifacts/contracts/Bridge.sol/Bridge.json';
import wiotaJson from '../../../solidity/artifacts/contracts/WIOTA.sol/WIOTA.json';

const wallet = new ethers.Wallet(bridgeOwnerSK, ethProvider);

// Connect to the bridge
const bridge = new ethers.Contract(bridgeAddress, bridgeJson.abi, wallet);

// Connect to the bridge
const wiota = new ethers.Contract(wiotaAddress, wiotaJson.abi, wallet);

/**
 * Calls the mintIOTA function on the smart contract to mint IOTA tokens.
 * @param {string} recipient - The recipient address.
 * @param {number} amount - The amount of IOTA tokens to mint.
 */
async function mintWIOTA(recipient: string, amount: number) {
    try {
        if (amount <= 0) {
            throw new Error("Amount must be greater than zero");
        }

        const tx = await bridge.mintWIOTA(recipient, amount);
        console.log(`Transaction sent: ${tx.hash}`);

        const receipt = await tx.wait();
        console.log(`Transaction confirmed, block number: ${receipt.blockNumber}`);
    } catch (error) {
        console.error(`Error minting WIOTA tokens: ${(error as Error).message}`);
    }
}

/**
 * Calls the unlockETH function on the smart contract.
 * @param {string} recipient - The address associated with the unlock event.
 * @param {number} amount - The amount of ethers to unlock.
 */
async function unlockETH(recipient: string, amount: number) {
    try {
        if (amount <= 0) {
            throw new Error("Amount must be greater than zero");
        }

        const tx = await bridge.unlockETH(
            recipient,
            amount,
        );
        console.log(`Transaction sent: ${tx.hash}`);

        const receipt = await tx.wait();
        console.log(`Transaction confirmed, block number: ${receipt.blockNumber}`);
    } catch (error) {
        console.error(`Error unlocking ETH tokens: ${(error as Error).message}`);
    }
}

async function getWiotaBalance(address: string) {
    const balance = await wiota.balanceOf(address);
    return balance;
}

export { mintWIOTA, unlockETH, getWiotaBalance };