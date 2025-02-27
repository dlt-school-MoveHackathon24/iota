// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { blake2b } from '@noble/hashes/blake2b';

import type { IotaClient } from '../client/index.js';
import type { Keypair } from '../cryptography/keypair.js';
import { toSerializedSignature } from '../cryptography/signature.js';
import type { SerializedSignature } from '../cryptography/signature.js';
import { SignerWithProvider } from './signer-with-provider.js';

export class RawSigner extends SignerWithProvider {
    private readonly keypair: Keypair;

    constructor(keypair: Keypair, client: IotaClient) {
        super(client);
        this.keypair = keypair;
    }

    async getAddress(): Promise<string> {
        return this.keypair.getPublicKey().toIotaAddress();
    }

    async signData(data: Uint8Array): Promise<SerializedSignature> {
        const pubkey = this.keypair.getPublicKey();
        const digest = blake2b(data, { dkLen: 32 });
        const signature = this.keypair.signData(digest);
        const signatureScheme = this.keypair.getKeyScheme();

        return toSerializedSignature({
            signatureScheme,
            signature,
            publicKey: pubkey,
        });
    }

    connect(client: IotaClient): SignerWithProvider {
        return new RawSigner(this.keypair, client);
    }
}
