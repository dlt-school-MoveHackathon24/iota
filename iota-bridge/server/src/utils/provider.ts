import { ethers } from 'ethers';
import { IotaClient } from '@iota/iota-sdk/client';

const ethProvider = new ethers.JsonRpcProvider("http://127.0.0.1:8545");

const iotaClient = new IotaClient({ url: "http://127.0.0.1:9000" });

export { ethProvider, iotaClient };
