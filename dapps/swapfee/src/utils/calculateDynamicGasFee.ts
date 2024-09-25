// src/utils/calculateDynamicGasFee.ts

import { Token } from './fetchUserTokens';

export const BASE_GAS_FEE_IOTA = 1; // Base gas fee in IOTA's smallest unit (integer)

export function calculateDynamicGasFee(token: Token): bigint {
  const exchangeRate = getMockExchangeRate(token.coinType);

  if (!exchangeRate) {
    throw new Error(`Exchange rate for token ${token.coinType} is undefined.`);
  }

  if (exchangeRate.rateInIota === 0 || exchangeRate.rateInToken === 0) {
    throw new Error(`Exchange rate values for token ${token.coinType} cannot be zero.`);
  }

  // Calculate gas fee using integer arithmetic
  // gasFeeInToken = BASE_GAS_FEE_IOTA * (rateInToken / rateInIota)
  // Since we can't do fractional division with integers, we multiply first and then divide
  const gasFeeInToken = (BigInt(BASE_GAS_FEE_IOTA) * BigInt(exchangeRate.rateInToken)) / BigInt(exchangeRate.rateInIota);

  return gasFeeInToken;
}

// Function to get mock exchange rate
function getMockExchangeRate(coinType: string): { rateInIota: number; rateInToken: number } | undefined {
  // Define integer exchange rates between tokens and IOTA
  const exchangeRates: Record<string, { rateInIota: number; rateInToken: number }> = {
    // 'CoinType': { rateInIota: x, rateInToken: y } meaning x IOTA = y Token
    '0x2::iota::IOTA': { rateInIota: 1, rateInToken: 1 }, // 1 IOTA = 1 IOTA
    '0x1::ctf_a::CTFA': { rateInIota: 1, rateInToken: 2 }, // 1 IOTA = 2 CTF A Coin
    '0x1::ctf_b::CTFB': { rateInIota: 1, rateInToken: 4 }, // 1 IOTA = 4 CTF B Coin
    '0x1::mint_coin::MintCoin': { rateInIota: 1, rateInToken: 5 },  // 1 IOTA = 5 Mint Coin
    '0x1::horse::HORSE': { rateInIota: 1, rateInToken: 10 }, // 1 IOTA = 10 Horse Tokens
    '0xc7edfd192a618b68897a0e0cf330e600027dd2bbc0f1be4c63685349da059b05::airdrop::AIRDROP' : { rateInIota: 1, rateInToken: 1 }, // 1 IOTA = 20 Airdrop Tokens
    '0x8e86df869286d8b181b7115cbfb94a2ef84d9a5fedab15495b2cb23adc1fddb6::mintcoin::MINTCOIN': { rateInIota: 1, rateInToken: 1 }, // 1 IOTA = 1 Mint Coin
    // Add other tokens as needed
  };

  return exchangeRates[coinType];
}
