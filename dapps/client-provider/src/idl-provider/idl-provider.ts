import { isObjId, ObjId } from "../types";
import { TransactionBlock } from '@iota/iota-sdk/transactions';
import { IotaClient } from '@iota/iota-sdk/client';
import { Ed25519Keypair } from '@iota/iota-sdk/keypairs/ed25519';

/**
 * Client provider
 * //TODO: describe the class
 */
export class ClientProvider<Idl extends Record<string, Record<string, unknown>>> {
    constructor(private config: {
        rpcUrl: string;
        package: string;
        module: string;
        signer: Ed25519Keypair
    }) {}

     /**
     * Proxy-based method to allow `co.getFlag()` style calls
     */
     public get contract() {
        return new Proxy(this, {
            get: (target, methodName: string) => {
                if (typeof target.invoke === 'function') {
                    return (params: any) => target.invoke(methodName as keyof Idl, params);
                }
                throw new Error(`Method ${methodName} is not a valid contract function`);
            }
        });
    }

    /**
     * TODO: describe the method
     * @param methodName
     * @param params
     * @returns
     * @throws
    */
    public async invoke<K extends keyof Idl>(methodName: K, params: Idl[K]) {
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

                    // TODO: add support for multiple types

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