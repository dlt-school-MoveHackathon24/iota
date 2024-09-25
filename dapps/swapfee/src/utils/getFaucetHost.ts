/**
 * @file getFaucetHost.ts
 * @summary Returns the faucet host for the given network.
 * @author
 *   - Developer: Silvio Meneguzzo
 * @date September 20, 2024
 */

export function getFaucetHost(network: string): string {
    switch (network) {
      case 'testnet':
        return 'https://faucet.testnet.iota.org/';
      case 'hackanet':
        return 'https://faucet.hackanet.iota.cafe/gas'; // Correct faucet URL for Hackanet
      // Add other networks as needed
      default:
        throw new Error(`Unknown network: ${network}`);
    }
  }
  