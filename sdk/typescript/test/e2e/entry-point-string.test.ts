// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it } from 'vitest';

import { TransactionBlock } from '../../src/transactions';
import { publishPackage, setup, TestToolbox } from './utils/setup';

describe('Test Move call with strings', () => {
    let toolbox: TestToolbox;
    let packageId: string;

    async function callWithString(str: string | string[], len: number, funcName: string) {
        const tx = new TransactionBlock();
        tx.moveCall({
            target: `${packageId}::entry_point_types::${funcName}`,
            arguments: [tx.pure(str), tx.pure(len)],
        });
        const result = await toolbox.client.signAndExecuteTransactionBlock({
            transactionBlock: tx,
            signer: toolbox.keypair,
            options: {
                showEffects: true,
            },
        });
        expect(result.effects?.status.status).toEqual('success');
    }

    beforeAll(async () => {
        toolbox = await setup();
        const packagePath =
            __dirname + '/../../../../crates/iota-core/src/unit_tests/data/entry_point_types';
        ({ packageId } = await publishPackage(packagePath));
    });

    it('Test ascii', async () => {
        const s = 'SomeString';
        await callWithString(s, s.length, 'ascii_arg');
    });

    it('Test utf8', async () => {
        const s = 'çå∞≠¢õß∂ƒ∫';
        const byte_len = 24;
        await callWithString(s, byte_len, 'utf8_arg');
    });

    it('Test string vec', async () => {
        const s1 = 'çå∞≠¢';
        const s2 = 'õß∂ƒ∫';
        const byte_len = 24;
        await callWithString([s1, s2], byte_len, 'utf8_vec_arg');
    });
});
