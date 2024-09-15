import { isObjId, ObjId } from "./types";
import { TransactionBlock } from '@iota/iota-sdk/transactions';
import { IotaClient } from '@iota/iota-sdk/client';
import { Ed25519Keypair } from '@iota/iota-sdk/keypairs/ed25519';

/**
 * Client provider
 * //TODO: describe the class
 */
export class ClientProvider<T extends Record<string, Record<string, unknown>>> {
    constructor(private config: {
        rpcUrl: string;
        package: string;
        module: string;
        signer: Ed25519Keypair
    }) { }

    /**
     * TODO: describe the method
     * @param methodName
     * @param params
     * @returns
     * @throws
    */
    public async invoke<K extends keyof T>(methodName: K, params: T[K]) {
        const txb = new TransactionBlock();

        txb.moveCall({
            target: `${this.config.package}::${this.config.module}::${methodName.toString()}`,
            arguments: [
                ...Object.entries(params).map(([field_name, field_value]) => {
                    if (isObjId(field_value)) {
                        const id = (field_value as ObjId).id;
                        return txb.object(id)//TODO: brutto, da fare meglio
                    }
                    if (typeof field_value === 'string') {
                        return txb.pure(field_value, 'string')//TODO: rimuovere depracated
                    }
                    if (typeof field_value === 'number') {
                        return txb.pure(field_value, 'u64')//TODO: rimuovere depracated
                    }

                    throw new Error(`Unknown type for ${field_name} with value ${field_value}`);
                }),
            ],
        })

        const iotaClient = new IotaClient({ url: this.config.rpcUrl });

        return await iotaClient.signAndExecuteTransactionBlock({
            transactionBlock: txb,
            signer: this.config.signer,
            requestType: 'WaitForLocalExecution',
            options: {
                showEffects: true,
            },
        })
    }
}