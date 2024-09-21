import { isObjId, ObjId } from "../types";
import { TransactionBlock } from '@iota/iota-sdk/transactions';
import { IotaClient } from '@iota/iota-sdk/client';
import { Ed25519Keypair } from '@iota/iota-sdk/keypairs/ed25519';

/**
/**
 * ClientProvider class
 * 
 * This class provides a client for interacting with Move contracts deployed on the Iota blockchain.
 * It facilitates the execution of contract functions by constructing transaction blocks, 
 * and sending them to the blockchain. The provider uses a type-safe approach for invoking methods 
 * defined in an IDL (Interface Definition Language), ensuring that the method names and parameters 
 * are validated against the contract interface.
 * 
 * @typeParam Idl - A generic type parameter that represents the IDL structure of the contract, 
 * with function names as keys and their parameters as values.
 */
export class ClientProvider<Idl extends Record<string, Record<string, unknown>>> {
    /**
     * Creates an instance of the ClientProvider.
     * 
     * @param config - Configuration object containing the RPC URL, package name, module name, 
     * and signer details (Ed25519Keypair) for interacting with the blockchain.
     */
    constructor(private config: {
        rpcUrl: string;
        package: string;
        module: string;
        signer: Ed25519Keypair
    }) {}

    /**
     * Invokes a contract method specified by `methodName` with the provided `params`.
     * Constructs a transaction block, attaches the necessary arguments, signs it with the specified signer, 
     * and executes it on the Iota blockchain.
     * 
     * @typeParam K - The type of the method name, constrained to keys of the IDL.
     * 
     * @param methodName - The name of the Move contract method to be invoked.
     * @param params - An object containing the parameters for the method invocation, 
     * validated against the IDL for type safety.
     * 
     * @returns The result of the transaction execution.
     * 
     * @throws Will throw an error if an unknown parameter type is encountered or if 
     * the transaction execution fails.
     */
    public async invoke<K extends keyof Idl>(methodName: K, params: Idl[K]) {
        const txb = new TransactionBlock();

        const a: keyof TransactionBlock['pure'] = 'u64';

        txb.moveCall({
            target: `${this.config.package}::${this.config.module}::${methodName.toString()}`,
            arguments: [
                ...Object.entries(params).map(([field_name, field_value]) => {
                    if (isObjId(field_value)) {
                        const id = (field_value as ObjId).id;
                        return txb.object(id)
                    }
                    if (typeof field_value === 'string') {
                        return txb.pure.address(field_value)
                    }
                    if (typeof field_value === 'number') {
                        return txb.pure.u64(field_value)
                    }
                    if (typeof field_value === 'bigint') {
                        return txb.pure.u64(field_value)
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