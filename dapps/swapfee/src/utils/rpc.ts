// src/utils/rpc.ts

import { IotaClient } from '@iota/iota-sdk/client';

// Initialize IotaClient with your custom API endpoint
export const client = new IotaClient({
  url: 'https://api.hackanet.iota.cafe/',
});
