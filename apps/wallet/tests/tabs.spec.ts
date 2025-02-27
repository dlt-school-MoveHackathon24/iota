// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { expect, test } from './fixtures';
import { createWallet } from './utils/auth';

test('Assets tab', async ({ page, extensionUrl }) => {
    await createWallet(page, extensionUrl);
    await page.getByRole('navigation').getByRole('link', { name: 'Assets' }).click();

    await expect(page.getByRole('main').getByRole('heading')).toHaveText(/Assets/);
});

test('Apps tab', async ({ page, extensionUrl }) => {
    await createWallet(page, extensionUrl);
    await page.getByRole('navigation').getByRole('link', { name: 'Apps' }).click();

    await expect(page.getByRole('main')).toHaveText(
        /Apps below are actively curated but do not indicate any endorsement or relationship with Iota Wallet. Please DYOR./i,
    );
});

test('Activity tab', async ({ page, extensionUrl }) => {
    await createWallet(page, extensionUrl);
    await page.getByRole('navigation').getByRole('link', { name: 'Activity' }).click();

    await expect(page.getByRole('main').getByRole('heading')).toHaveText(/Your Activity/);
});
