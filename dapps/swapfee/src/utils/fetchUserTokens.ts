/**
 * @file fetchUserTokens.ts
 * @summary Fetches the user's tokens from the DLT.
 * @author
 *   - Developer: Silvio Meneguzzo
 * @date September 20, 2024
 */


import { client } from './rpc';
import { whitelistedCoinTypes } from './whitelistedCoinTypes';

export interface Token {
  coinId: string;
  balance: bigint;
  tokenName: string;
  coinType: string;
}

export async function fetchUserTokens(address: string): Promise<Token[]> {
  try {
    const tokens: Token[] = [];

    // Iterate over each coin type in the whitelist
    for (const coinType of whitelistedCoinTypes) {
      let nextCursor: string | null | undefined = null;
      do {
        const coinsResponse = await client.getCoins({
          owner: address,
          coinType: coinType,
          cursor: nextCursor,
          limit: 50, // Adjust limit as needed
        });

        for (const coin of coinsResponse.data) {
          const coinId = coin.coinObjectId;
          const balanceAmount = BigInt(coin.balance);

          // Since we don't have metadata, we'll use the coinType as the token name
          const tokenName = coinType;

          const token: Token = {
            coinId,
            balance: balanceAmount,
            tokenName,
            coinType,
          };

          tokens.push(token);
        }

        nextCursor = coinsResponse.nextCursor;
      } while (nextCursor);
    }

    return tokens;
  } catch (error) {
    console.error('Error fetching user tokens:', error);
    throw error;
  }
}
