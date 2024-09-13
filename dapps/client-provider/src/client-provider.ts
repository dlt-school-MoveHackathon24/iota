import { TransactionBlock } from '@iota/iota-sdk/transactions';
import { isObjId } from "./types";
import { IotaClient } from '@iota/iota-sdk/client';
import { Ed25519Keypair } from '@iota/iota-sdk/keypairs/ed25519';
import { getFaucetHost, requestIotaFromFaucetV0 } from '@iota/iota-sdk/faucet';

type Config = {
    rpcUrl: string;
    package: string;
    module: string;
    privateKey: string
}

export class ClientProvider<T extends Record<string, Record<string, unknown>>> {
    private config: Config;

    constructor(config: Config) {
        this.config = config;
    }

    async invoke<K extends keyof T>(
        methodName: K,
        params: T[K]
    ): Promise<void> {

        const iotaClient = new IotaClient({ url: this.config.rpcUrl });

        const txb = new TransactionBlock();

        txb.moveCall({
            target: `${this.config.package}::${this.config.module}::${methodName as string}`,
            arguments: [

            ],
        })

        const keypair = new Ed25519Keypair();

        await requestIotaFromFaucetV0({
            host: "https://faucet.hackanet.iota.cafe/gas",
            recipient: keypair.toIotaAddress(),
        });

        const result = await iotaClient.signAndExecuteTransactionBlock({
            transactionBlock: txb,
            signer: keypair,
            requestType: 'WaitForLocalExecution',
            options: {
                showEffects: true,
            },
        })

        txb.moveCall({
            target: `${this.config.package}::${this.config.module}::${methodName as string}`,
            arguments: [

            ],
        });

        console.log("Invoking", methodName, params);
        Object.entries(params).forEach(([field_name, field_value]) => {
            console.log(field_name, field_value);
            if (isObjId(field_value)) {
                console.log("Is ObjId");
            }
            if (typeof field_value === 'string') {
                console.log("Is string");
            }
            if (typeof field_value === 'number') {
                console.log("Is number");
            }
        });
    }

}
