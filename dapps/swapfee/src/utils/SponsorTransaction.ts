// src/utils/SponsorTransaction.ts

import { IotaObjectRef } from '@iota/iota-sdk/client';
import { getFaucetRequestStatus, requestIotaFromFaucetV1 } from '@iota/iota-sdk/faucet';
import { Ed25519Keypair } from '@iota/iota-sdk/keypairs/ed25519';
import { TransactionBlock } from '@iota/iota-sdk/transactions';
import { client } from './rpc';
import { getFaucetHost } from './getFaucetHost'; // Ensure correct import path

export async function sponsorTransaction(
  sender: string,
  transactionKindBytes: Uint8Array,
  isGasPayment: boolean = false
): Promise<any> {
  const keypair = new Ed25519Keypair();
  const sponsorAddress = keypair.getPublicKey().toIotaAddress();
  console.log(`Sponsor address: ${sponsorAddress}`);

  // Fund the sponsor account
  const faucetHost = getFaucetHost('hackanet'); // Get the faucet URL for hackanet

  // Request funds from the faucet
  await requestTokensFromFaucet(sponsorAddress, faucetHost);

  // Wait for funds to arrive
  let gasPayment: IotaObjectRef[] = [];
  let retries = 50;
  while (retries !== 0) {
    const coins = await client.getCoins({ owner: sponsorAddress, limit: 1 });
    if (coins.data.length > 0) {
      gasPayment = coins.data.map((coin) => ({
        objectId: coin.coinObjectId,
        version: coin.version,
        digest: coin.digest,
      }));
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 2000)); // Increased delay to 2 seconds
    retries -= 1;
  }

  if (gasPayment.length === 0) {
    throw new Error('Sponsor account did not receive funds from faucet');
  }

  const tx = TransactionBlock.fromKind(transactionKindBytes);
  tx.setSender(sender);
  tx.setGasOwner(sponsorAddress);
  tx.setGasPayment(gasPayment);

  if (isGasPayment) {
    // For the gas payment transaction, no additional verification is needed here
    // The service will verify its receipt before sponsoring the main transaction
  } else {
    // Verify that the gas payment has been made
    const gasPaymentVerified = await verifyGasPayment(sender);
    if (!gasPaymentVerified) {
      throw new Error('Gas payment not verified for sender: ' + sender);
    }
  }

  // Sign the transaction block with the sponsor's keypair
  const signedTx = await keypair.signTransactionBlock(await tx.build({ client }));
  return signedTx;
}

// Function to request tokens from the faucet
async function requestTokensFromFaucet(address: string, host: string): Promise<void> {
  const { error, task: taskId } = await requestIotaFromFaucetV1({
    recipient: address,
    host,
  });

  if (error || !taskId) {
    throw new Error(error ?? 'Failed, task id not found.');
  }

  let currentStatus = 'INPROGRESS';
  let requestStatusCount = 0;
  const MAX_FAUCET_REQUESTS_STATUS = 20;
  const FAUCET_REQUEST_DELAY = 1500;

  while (currentStatus === 'INPROGRESS') {
    const {
      status: { status },
      error,
    } = await getFaucetRequestStatus({
      host,
      taskId,
    });

    currentStatus = status;

    if (
      currentStatus === 'DISCARDED' ||
      error ||
      requestStatusCount > MAX_FAUCET_REQUESTS_STATUS
    ) {
      throw new Error(error ?? status ?? 'Something went wrong with the faucet request');
    }

    if (currentStatus === 'SUCCEEDED') {
      console.log(`Faucet request succeeded for address: ${address}`);
      return;
    }
    requestStatusCount += 1;
    await new Promise((resolve) => setTimeout(resolve, FAUCET_REQUEST_DELAY));
  }

  throw new Error('Something went wrong with the faucet request');
}

// Function to verify gas payment
async function verifyGasPayment(sender: string): Promise<boolean> {
  // Implement logic to verify the gas payment transaction
  // For example, check if the expected amount of custom tokens has been received from the sender
  console.log('Verifying gas payment for sender:', sender);

  // TODO: Implement actual verification logic
  // This may involve querying the blockchain for transactions from the sender to the service's address

  return true; // Placeholder return value
}
