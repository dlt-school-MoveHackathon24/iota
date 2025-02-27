// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, expect, it } from 'vitest';

import { setup, TestToolbox } from './utils/setup';

describe('Object id/Address/Transaction digest validation', () => {
    let toolbox: TestToolbox;

    beforeAll(async () => {
        toolbox = await setup();
    });

    //Test that with invalid object id/address/digest, functions will throw an error before making a request to the rpc server
    it('Test all functions with invalid Iota Address', async () => {
        //empty id
        expect(toolbox.client.getOwnedObjects({ owner: '' })).rejects.toThrowError(
            /Invalid Iota address/,
        );
    });

    it('Test all functions with invalid Object Id', async () => {
        //empty id
        expect(toolbox.client.getObject({ id: '' })).rejects.toThrowError(/Invalid Iota Object id/);

        //more than 20bytes
        expect(
            toolbox.client.getDynamicFields({
                parentId: '0x0000000000000000000000004ce52ee7b659b610d59a1ced129291b3d0d4216322',
            }),
        ).rejects.toThrowError(/Invalid Iota Object id/);

        //wrong batch request
        const objectIds = ['0xBABE', '0xCAFE', '0xWRONG', '0xFACE'];
        expect(toolbox.client.multiGetObjects({ ids: objectIds })).rejects.toThrowError(
            /Invalid Iota Object id 0xWRONG/,
        );
    });

    it('Test all functions with invalid Transaction Digest', async () => {
        //empty digest
        expect(toolbox.client.getTransactionBlock({ digest: '' })).rejects.toThrowError(
            /Invalid Transaction digest/,
        );

        //wrong batch request
        const digests = ['AQ7FA8JTGs368CvMkXj2iFz2WUWwzP6AAWgsLpPLxUmr', 'wrong'];
        expect(toolbox.client.multiGetTransactionBlocks({ digests })).rejects.toThrowError(
            /Invalid Transaction digest wrong/,
        );
    });
});
