import * as fs from 'fs';
import { ClientProvider } from "./client-provider";
import { CtfLuckynumberInterface } from "./CtfLuckynumberInterface";
import { requestIotaFromFaucetV0 } from '@iota/iota-sdk/faucet';
import { Ed25519Keypair } from '@iota/iota-sdk/keypairs/ed25519';

/**
 * Main function for testing the client provider
 */
async function main() {
    const passphrase = fs.readFileSync("./passphrase.sk").toString('utf-8') // TODO: add to .env
    const signer = Ed25519Keypair.deriveKeypair(passphrase)

    await requestIotaFromFaucetV0({
        host: "https://faucet.hackanet.iota.cafe/gas",//TODO: add to .env
        recipient: signer.toIotaAddress(),
    })

    const cp = new ClientProvider<CtfLuckynumberInterface>({
        rpcUrl: "https://api.hackanet.iota.cafe/",//TODO: add to .env
        package: "0x699171bd7ce9062b07716e08068d27350d6d7c1f40a141ae0774f962f3d52f33",//TODO: make a variable
        module: "luckynumber",//TODO: make a variable
        signer
    });

    const response = await cp.invoke("get_flag", { user_counter: { id: "0x02e95725b95a33992591bdeebe4fa84ce9570faf8808b58b1760229f83064043" }, lucky_num: 1 })
    console.log(response)
}

main().then(
    () => process.exit(),
    err => {
        console.error(err);
        process.exit(-1);
    }
);