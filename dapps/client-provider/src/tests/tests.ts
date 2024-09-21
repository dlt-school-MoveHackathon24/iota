import dotenv from 'dotenv';

import { ClientProvider } from "../idl-provider/idl-provider";
import { CtfLuckynumberIdl, moduleName } from "../idl/CtfLuckynumberIdl";

import { requestIotaFromFaucetV0 } from '@iota/iota-sdk/faucet';
import { Ed25519Keypair } from '@iota/iota-sdk/keypairs/ed25519';

/**
 * Main function for testing the client provider
 */
async function main() {
    // Load environment variables from .env file
    dotenv.config();

    if(!process.env.IOTA_CP_PASSPHRASE || !process.env.IOTA_CP_RPC || !process.env.IOTA_CP_FAUCET) 
        throw new Error("Error while loading the environment variables. Make sure a correct .env file exists.");

    // Just generating a keypair for the test
    const passphrase = process.env.IOTA_CP_PASSPHRASE;
    const signer = Ed25519Keypair.deriveKeypair(passphrase)

    await requestIotaFromFaucetV0({
        host: process.env.IOTA_CP_FAUCET,
        recipient: signer.toIotaAddress(),
    })

    // Building an Iota client provider to interact with the smart contract module
    const cp = new ClientProvider<CtfLuckynumberIdl>({
        rpcUrl: process.env.IOTA_CP_RPC,
        package: "0x119933ce0b523bfda93d677b9024991f4f084d2d5fa3657c256a689fe572bdaf",// Your package address
        module: moduleName,
        signer
    });

    // Calling the get_flag function from the smart contract (challenge 2 of the CTF)
    const response = await cp.invoke("get_flag", { 
        user_counter: { id: "0x0a9a9c966c3022938ca8e63997da4c21d144bbd6561aa318ea84364f6359407f" },
        lucky_num: 1 
    })

    console.log(response)
}

main().then(
    () => process.exit(),
    err => {
        console.error(err);
        process.exit(-1);
    }
);