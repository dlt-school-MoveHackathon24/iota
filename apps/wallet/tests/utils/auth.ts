// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type { Page } from '@playwright/test';

export const PASSWORD = 'iota';

export async function createWallet(page: Page, extensionUrl: string) {
    await page.goto(extensionUrl);
    await page.getByRole('link', { name: /More Options/ }).click();
    await page.getByRole('link', { name: /Create a new Passphrase Account/ }).click();
    await page.getByLabel('Create Account Password').fill('iota');
    await page.getByLabel('Confirm Account Password').fill('iota');
    await page.getByLabel('I read and agreed to the').click();
    await page.getByRole('button', { name: /Create Wallet/ }).click();
    await page.locator('label', { has: page.locator('input[type=checkbox]') }).click();
    await page.getByRole('link', { name: /Open Iota Wallet/ }).click();
}

export async function importWallet(page: Page, extensionUrl: string, mnemonic: string | string[]) {
    await page.goto(extensionUrl);
    await page.getByRole('link', { name: /More Options/ }).click();
    await page.getByRole('link', { name: /Import Passphrase/ }).click();
    await page
        .getByPlaceholder('Password')
        .first()
        .type(typeof mnemonic === 'string' ? mnemonic : mnemonic.join(' '));
    await page.getByRole('button', { name: /Add Account/ }).click();
    await page.getByLabel('Create Account Password').fill(PASSWORD);
    await page.getByLabel('Confirm Account Password').fill(PASSWORD);
    await page.getByLabel('I read and agreed to the').click();
    await page.getByRole('button', { name: /Create Wallet/ }).click();
}
