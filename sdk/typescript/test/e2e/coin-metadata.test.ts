// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { beforeEach, describe, expect, it } from 'vitest';

import { publishPackage, setup, TestToolbox } from './utils/setup';

describe('Test Coin Metadata', () => {
    let toolbox: TestToolbox;
    let packageId: string;

    beforeEach(async () => {
        toolbox = await setup();
        const packagePath = __dirname + '/./data/coin_metadata';
        ({ packageId } = await publishPackage(packagePath));
    });

    it('Test accessing coin metadata', async () => {
        const coinMetadata = (await toolbox.client.getCoinMetadata({
            coinType: `${packageId}::test::TEST`,
        }))!;
        expect(coinMetadata.decimals).to.equal(2);
        expect(coinMetadata.name).to.equal('Test Coin');
        expect(coinMetadata.description).to.equal('Test coin metadata');
        expect(coinMetadata.iconUrl).to.equal('http://iota.io');
    });
});
