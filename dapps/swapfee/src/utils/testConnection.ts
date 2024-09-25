/**
 * @file testConnection.ts
 * @summary Tests the connection to the IOTA network. If successful, it logs the chain identifier.
 * @author
 *   - Developer: Silvio Meneguzzo
 * @date September 20, 2024
 */

import { client } from './rpc';

async function testConnection() {
  try {
    const info = await client.getChainIdentifier();
    console.log('Connected to network:', info);
  } catch (error) {
    console.error('Error connecting to network:', error);
  }
}

testConnection();
