import * as fs from 'fs';
import dotenv from 'dotenv';

import { ClientProvider } from "../idl-provider/idl-provider";
import { CtfLuckynumberIdl, moduleName } from "../../CtfLuckynumberIdl";

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

    const passphrase = process.env.IOTA_CP_PASSPHRASE;
    const signer = Ed25519Keypair.deriveKeypair(passphrase)

    await requestIotaFromFaucetV0({
        host: process.env.IOTA_CP_FAUCET,
        recipient: signer.toIotaAddress(),
    })

    const cp = new ClientProvider<CtfLuckynumberIdl>({
        rpcUrl: process.env.IOTA_CP_RPC,
        package: "0x699171bd7ce9062b07716e08068d27350d6d7c1f40a141ae0774f962f3d52f33", //TODO: make a variable
        module: moduleName,
        signer
    });

    const response = await cp.invoke("get_flag", { 
        user_counter: { id: "0x02e95725b95a33992591bdeebe4fa84ce9570faf8808b58b1760229f83064043" }, 
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