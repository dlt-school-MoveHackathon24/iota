// src/utils/testConnection.ts

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
