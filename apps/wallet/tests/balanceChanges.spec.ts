// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { expect, test } from './fixtures';
import { createWallet, importWallet } from './utils/auth';
import { generateKeypairFromMnemonic, requestIotaFromFaucet } from './utils/localnet';

const receivedAddressMnemonic = [
    'beef',
    'beef',
    'beef',
    'beef',
    'beef',
    'beef',
    'beef',
    'beef',
    'beef',
    'beef',
    'beef',
    'beef',
];

const currentWalletMnemonic = [
    'intact',
    'drift',
    'gospel',
    'soft',
    'state',
    'inner',
    'shed',
    'proud',
    'what',
    'box',
    'bean',
    'visa',
];

const COIN_TO_SEND = 20;

test('request IOTA from local faucet', async ({ page, extensionUrl }) => {
    const timeout = 30_000;
    test.setTimeout(timeout);
    await createWallet(page, extensionUrl);
    await page.getByRole('navigation').getByRole('link', { name: 'Home' }).click();

    const originalBalance = await page.getByTestId('coin-balance').textContent();
    await page.getByTestId('faucet-request-button').click();
    await expect(page.getByText(/IOTA Received/i)).toBeVisible({ timeout });
    await expect(page.getByTestId('coin-balance')).not.toHaveText(`${originalBalance}IOTA`);
});

test('send 20 IOTA to an address', async ({ page, extensionUrl }) => {
    const receivedKeypair = await generateKeypairFromMnemonic(receivedAddressMnemonic.join(' '));
    const receivedAddress = receivedKeypair.getPublicKey().toIotaAddress();

    const originKeypair = await generateKeypairFromMnemonic(currentWalletMnemonic.join(' '));
    const originAddress = originKeypair.getPublicKey().toIotaAddress();

    await importWallet(page, extensionUrl, currentWalletMnemonic);
    await page.getByRole('navigation').getByRole('link', { name: 'Home' }).click();

    await requestIotaFromFaucet(originAddress);
    await expect(page.getByTestId('coin-balance')).not.toHaveText('0IOTA');

    const originalBalance = await page.getByTestId('coin-balance').textContent();

    await page.getByTestId('send-coin-button').click();
    await page.getByTestId('coin-amount-input').fill(String(COIN_TO_SEND));
    await page.getByTestId('address-input').fill(receivedAddress);
    await page.getByRole('button', { name: 'Review' }).click();
    await page.getByRole('button', { name: 'Send Now' }).click();
    await expect(page.getByTestId('overlay-title')).toHaveText('Transaction');

    await page.getByTestId('close-icon').click();
    await page.getByTestId('nav-tokens').click();
    await expect(page.getByTestId('coin-balance')).not.toHaveText(`${originalBalance}IOTA`);
});

test('check balance changes in Activity', async ({ page, extensionUrl }) => {
    const originKeypair = await generateKeypairFromMnemonic(currentWalletMnemonic.join(' '));
    const originAddress = originKeypair.getPublicKey().toIotaAddress();

    await importWallet(page, extensionUrl, currentWalletMnemonic);
    await page.getByRole('navigation').getByRole('link', { name: 'Home' }).click();

    await requestIotaFromFaucet(originAddress);
    await page.getByTestId('nav-activity').click();
    await page
        .getByText(/Transaction/i)
        .first()
        .click();
    await expect(page.getByText(`${COIN_TO_SEND} IOTA`, { exact: false })).toBeVisible();
});
