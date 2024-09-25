// src/utils/rpc.ts
/**
 * @file rpc.ts
 * @summary Initializes the IotaClient with the custom API endpoint.
 * @author
 *   - Developer: Silvio Meneguzzo
 * @date September 20, 2024
 */


import { IotaClient } from '@iota/iota-sdk/client';

// Initialize IotaClient with your custom API endpoint
export const client = new IotaClient({
  url: 'https://api.hackanet.iota.cafe/',
});
