// src/App.tsx

import React, { useState, useEffect } from 'react';
import {
  useCurrentAccount,
  ConnectButton,
  useSignTransactionBlock,
  useIotaClient,
} from '@iota/dapp-kit';
import { TransactionBlock } from '@iota/iota-sdk/transactions';
import { sponsorTransaction } from './utils/SponsorTransaction';
import { calculateDynamicGasFee } from './utils/calculateDynamicGasFee';
import { fetchUserTokens, Token } from './utils/fetchUserTokens';
import './styles/App.css';

export interface MainTxDetails {
  tokenToSend: Token | null;
  recipientAddress: string;
  amountToSend: string;
}

export function App() {
  const client = useIotaClient();
  const { mutateAsync: signTransactionBlock } = useSignTransactionBlock();
  const [loading, setLoading] = useState(false);

  // Declare currentAccount
  const currentAccount = useCurrentAccount();

  // State variables
  const [userTokens, setUserTokens] = useState<Token[]>([]);
  const [selectedToken, setSelectedToken] = useState<Token | null>(null);
  const [tokenAmount, setTokenAmount] = useState<bigint>(BigInt(0));
  const [servicePaymentTx, setServicePaymentTx] = useState<any>(null);
  const [servicePaymentExecuted, setServicePaymentExecuted] = useState(false);
  const [mainTx, setMainTx] = useState<any>(null);
  const [mainTxExecuted, setMainTxExecuted] = useState(false);
  const [executedTx, setExecutedTx] = useState<any>(null);

  // Step management
  const [step, setStep] = useState<
    'tokenSelection' | 'servicePayment' | 'mainTransaction' | 'review' | 'transaction' | 'completed'
  >('tokenSelection');

  // State variables for main transaction
  const [mainTxDetails, setMainTxDetails] = useState<MainTxDetails>({
    tokenToSend: null,
    recipientAddress: '',
    amountToSend: '',
  });

  // Token name mapping
  const tokenNameMapping: Record<string, string> = {
    '0x2::iota::IOTA': 'IOTA',
    '0x1::ctf_a::CTFA': 'CTF A Coin',
    '0x1::ctf_b::CTFB': 'CTF B Coin',
    '0x1::mint_coin::MintCoin': 'Mint Coin',
    '0x1::horse::HORSE': 'Horse Tokens',
    // Add other mappings as needed
  };

  // Function to get the token name
  const getTokenName = (coinType: string): string => {
    return tokenNameMapping[coinType] || 'Unknown Token';
  };

  // Fetch user's tokens when the wallet connects
  const fetchTokens = async () => {
    if (!currentAccount?.address) return;
    setLoading(true);
    try {
      const tokens = await fetchUserTokens(currentAccount!.address);
      setUserTokens(tokens);
      if (tokens.length > 0) {
        setStep('tokenSelection');
      } else {
        alert('No tokens found in the connected wallet.');
      }
    } catch (error) {
      console.error('Error fetching user tokens:', error);
      alert('Failed to fetch tokens. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  // Call fetchTokens when the wallet connects or currentAccount changes
  useEffect(() => {
    fetchTokens();
  }, [currentAccount]);

  // Handle token selection for service payment
  const handleTokenSelect = (token: Token) => {
    setLoading(true);
    try {
      const amount = calculateDynamicGasFee(token);
      setSelectedToken(token);
      setTokenAmount(amount);
      setStep('servicePayment'); // Proceed to service payment step
    } catch (error: any) {
      console.error('Error calculating gas fee:', error);
      alert(error.message);
    } finally {
      setLoading(false);
    }
  };

  // Create the service payment transaction
  const createServicePaymentTransaction = async () => {
    if (!currentAccount?.address) {
      alert('Please connect your wallet first.');
      return;
    }

    setLoading(true);
    try {
      const tx = new TransactionBlock();

      const serviceAddress = '0x4dafe69422fccffdd3804d76077f55060002c8e4a4b156b5cd79712b9cf92a4b'; // Replace with your service's address

      const paymentToken = selectedToken;
      if (!paymentToken) {
        alert('No token selected for payment.');
        setLoading(false);
        return;
      }

      const paymentAmount = tokenAmount;

      // Ensure the user has enough balance
      if (paymentAmount > paymentToken.balance) {
        alert('Insufficient balance to send the payment.');
        setLoading(false);
        return;
      }

      const coinObject = tx.object(paymentToken.coinId);

      if (paymentAmount === paymentToken.balance) {
        // Transfer the entire coin to the service
        tx.transferObjects([coinObject], tx.pure(serviceAddress));
      } else {
        // Split off the payment amount
        const [paymentCoin] = tx.splitCoins(coinObject, [tx.pure(paymentAmount.toString())]);

        // Transfer the paymentCoin to the service
        tx.transferObjects([paymentCoin], tx.pure(serviceAddress));

        // Transfer the remaining balance (coinObject) back to the user
        tx.transferObjects([coinObject], tx.pure(currentAccount!.address));
      }

      // Build the transaction bytes
      const paymentTxBytes = await tx.build({ client, onlyTransactionKind: true });

      // Use sponsorTransaction to sponsor the transaction
      const sponsoredPaymentTx = await sponsorTransaction(
        currentAccount!.address,
        paymentTxBytes,
        false // isGasPayment
      );

      // Store the sponsored transaction
      setServicePaymentTx(sponsoredPaymentTx);

      // Proceed to transaction step
      setStep('transaction');
    } catch (error) {
      console.error('Error creating service payment transaction:', error);
      alert('Failed to create service payment transaction.');
    } finally {
      setLoading(false);
    }
  };

  // Sign and execute the service payment transaction
  const signAndExecuteServicePayment = async () => {
    if (!servicePaymentTx) return;
    setLoading(true);
    try {
      const signedServicePaymentTx = await signTransactionBlock({
        transactionBlock: TransactionBlock.from(servicePaymentTx.bytes),
      });

      const executedServicePayment = await client.executeTransactionBlock({
        transactionBlock: signedServicePaymentTx.transactionBlockBytes,
        signature: [signedServicePaymentTx.signature, servicePaymentTx.signature],
        options: {
          showEffects: true,
          showEvents: true,
          showObjectChanges: true,
        },
      });

      if (executedServicePayment.effects?.status?.status === 'success') {
        setServicePaymentExecuted(true);
        // Proceed to main transaction step
        setStep('mainTransaction');
      } else {
        console.error('Service payment transaction failed:', executedServicePayment.effects?.status);
        alert('Service payment transaction failed.');
      }
    } catch (error) {
      console.error('Error executing service payment transaction:', error);
      alert('Failed to execute service payment transaction.');
    } finally {
      setLoading(false);
    }
  };

  // Handle main transaction details input
  const handleMainTxDetailsSubmit = () => {
    if (
      !mainTxDetails.tokenToSend ||
      !mainTxDetails.recipientAddress ||
      !mainTxDetails.amountToSend
    ) {
      alert('Please fill in all fields.');
      return;
    }

    // Validate that amountToSend is a positive integer
    if (!/^\d+$/.test(mainTxDetails.amountToSend) || parseInt(mainTxDetails.amountToSend) <= 0) {
      alert('Please enter a valid positive integer amount to send.');
      return;
    }

    setStep('review');
  };

  // Create main transaction
  const createMainTransaction = async () => {
    if (!currentAccount?.address) {
      alert('Please connect your wallet first.');
      return;
    }

    setLoading(true);
    try {
      const tx = new TransactionBlock();

      // Transfer the specified amount of the token to the recipient
      const tokenToSend = userTokens.find(
        (token) => token.coinId === mainTxDetails.tokenToSend?.coinId
      );

      if (!tokenToSend) {
        alert('Selected token not found in your wallet.');
        setLoading(false);
        return;
      }

      // Use the amount to send (integer)
      const amountToSend = BigInt(mainTxDetails.amountToSend);

      if (amountToSend > tokenToSend.balance) {
        alert('Insufficient balance to send the specified amount.');
        setLoading(false);
        return;
      }

      const coinObject = tx.object(tokenToSend.coinId);

      if (amountToSend === tokenToSend.balance) {
        // Transfer the entire coin
        tx.transferObjects([coinObject], tx.pure(mainTxDetails.recipientAddress));
      } else {
        // Split off the amount to send
        const [amountToSendCoin] = tx.splitCoins(coinObject, [tx.pure(amountToSend.toString())]);

        // Transfer the amountToSendCoin to the recipient
        tx.transferObjects([amountToSendCoin], tx.pure(mainTxDetails.recipientAddress));

        // Transfer the remaining balance (coinObject) back to the user
        tx.transferObjects([coinObject], tx.pure(currentAccount!.address));
      }

      const mainTxBytes = await tx.build({ client, onlyTransactionKind: true });

      const sponsoredMainTx = await sponsorTransaction(
        currentAccount!.address,
        mainTxBytes,
        false // isGasPayment
      );

      setMainTx(sponsoredMainTx);
    } catch (error: any) {
      console.error('Error creating main transaction:', error);
      alert('Failed to create main transaction.');
    } finally {
      setLoading(false);
    }
  };

  // Sign and execute main transaction
  const signAndExecuteMainTransaction = async () => {
    if (!mainTx) return;
    setLoading(true);
    try {
      const signedMainTx = await signTransactionBlock({
        transactionBlock: TransactionBlock.from(mainTx.bytes),
      });

      const executedMainTx = await client.executeTransactionBlock({
        transactionBlock: signedMainTx.transactionBlockBytes,
        signature: [signedMainTx.signature, mainTx.signature],
        options: {
          showEffects: true,
          showEvents: true,
          showObjectChanges: true,
        },
      });

      if (executedMainTx.effects?.status?.status === 'success') {
        setMainTxExecuted(true);
        setExecutedTx(executedMainTx);
        setStep('completed');
      } else {
        console.error('Main transaction failed:', executedMainTx.effects?.status);
        alert('Main transaction failed.');
      }
    } catch (error) {
      console.error('Error executing main transaction:', error);
      alert('Failed to execute main transaction.');
    } finally {
      setLoading(false);
    }
  };

  // Handle back navigation
  const goBack = () => {
    if (step === 'servicePayment') {
      setStep('tokenSelection');
    } else if (step === 'mainTransaction') {
      setStep('servicePayment');
    } else if (step === 'review') {
      setStep('mainTransaction');
    } else if (step === 'transaction') {
      setStep('review');
    }
  };

  // Render component
  return (
    <div className="p-8">
      {/* Wallet Connection */}
      <div className="mb-6">
        <ConnectButton className="!bg-indigo-600 !text-white" />
      </div>

      {/* Display Connected Account Address */}
      {currentAccount?.address && (
        <div className="mb-6">
          <p>
            <strong>Connected Address:</strong> {currentAccount.address}
          </p>
        </div>
      )}

      {/* Main Steps */}
      {step === 'tokenSelection' && (
        <div>
          <h2>Select a Token to Pay for Gas Fees</h2>
          {loading && <p>Loading...</p>}
          {userTokens.length === 0 && (
            <div>
              <p>No tokens found in the connected wallet.</p>
              <Button onClick={fetchTokens} disabled={loading}>
                Refresh Tokens
              </Button>
            </div>
          )}
          {userTokens.length > 0 && (
            <div className="grid grid-cols-1 gap-4">
              {userTokens.map((token) => (
                <div key={token.coinId} className="border p-4 rounded-lg shadow-md">
                  <p className="font-semibold text-lg">{getTokenName(token.coinType)}</p>
                  <p className="text-sm text-gray-600">Coin Type: {token.coinType}</p>
                  <p className="text-sm text-gray-600">Coin ID: {token.coinId}</p>
                  <p className="text-sm text-gray-600">Balance: {token.balance.toString()}</p>
                  <Button onClick={() => handleTokenSelect(token)} disabled={loading}>
                    Select for Gas Payment
                  </Button>
                </div>
              ))}
            </div>
          )}
          <Button onClick={goBack} disabled={loading}>
            Back
          </Button>
        </div>
      )}

      {step === 'servicePayment' && (
        <div>
          <h2>Service Payment</h2>
          <p>
            You need to send{' '}
            <strong>
              {tokenAmount.toString()} {getTokenName(selectedToken?.coinType || '')}
            </strong>{' '}
            to the service to proceed.
          </p>
          <Button onClick={createServicePaymentTransaction} disabled={loading}>
            Proceed with Service Payment
          </Button>
          <Button onClick={goBack} disabled={loading}>
            Back
          </Button>
          {loading && <p>Loading...</p>}
        </div>
      )}

      {step === 'mainTransaction' && (
        <div>
          <h2>Specify Main Transaction</h2>
          <label className="block mb-2">Select Token to Send:</label>
          <select
            value={mainTxDetails.tokenToSend?.coinId || ''}
            onChange={(e) => {
              const token = userTokens.find((t) => t.coinId === e.target.value);
              setMainTxDetails({ ...mainTxDetails, tokenToSend: token || null });
            }}
            className="border p-2 w-full mb-4"
          >
            <option value="">Select a token</option>
            {userTokens.map((token) => (
              <option key={token.coinId} value={token.coinId}>
                {getTokenName(token.coinType)} ({token.balance.toString()})
              </option>
            ))}
          </select>

          <label className="block mb-2">Recipient Address:</label>
          <input
            type="text"
            value={mainTxDetails.recipientAddress}
            onChange={(e) =>
              setMainTxDetails({ ...mainTxDetails, recipientAddress: e.target.value })
            }
            placeholder="Enter recipient's IOTA address"
            className="border p-2 w-full mb-4"
          />

          <label className="block mb-2">Amount to Send:</label>
          <input
            type="text"
            value={mainTxDetails.amountToSend}
            onChange={(e) =>
              setMainTxDetails({ ...mainTxDetails, amountToSend: e.target.value })
            }
            placeholder="Enter amount (integer)"
            className="border p-2 w-full mb-4"
          />

          <Button onClick={handleMainTxDetailsSubmit} disabled={loading}>
            Continue
          </Button>
          <Button onClick={goBack} disabled={loading}>
            Back
          </Button>
        </div>
      )}

      {step === 'review' && (
        <div>
          <h2>Review and Confirm</h2>
          <h3>Service Payment</h3>
          <p>
            You will send{' '}
            <strong>
              {tokenAmount.toString()} {getTokenName(selectedToken?.coinType || '')}
            </strong>{' '}
            to the service.
          </p>
          <h3>Main Transaction</h3>
          <p>
            You will send{' '}
            <strong>
              {mainTxDetails.amountToSend} {getTokenName(mainTxDetails.tokenToSend?.coinType || '')}
            </strong>{' '}
            to <strong>{mainTxDetails.recipientAddress}</strong>.
          </p>
          <Button
            onClick={() => {
              createMainTransaction();
              setStep('transaction');
            }}
            disabled={loading}
          >
            Proceed with Transactions
          </Button>
          <Button onClick={goBack} disabled={loading}>
            Back
          </Button>
          {loading && <p>Loading...</p>}
        </div>
      )}

      {step === 'transaction' && (
        <div>
          <h2>Sign and Execute Transactions</h2>
          {loading && <p>Loading...</p>}
          {!servicePaymentExecuted && (
            <div>
              <h3>Service Payment Transaction</h3>
              <p>Please sign the service payment transaction using your wallet.</p>
              <Button onClick={signAndExecuteServicePayment} disabled={loading}>
                Sign and Execute Service Payment Transaction
              </Button>
            </div>
          )}
          {servicePaymentExecuted && !mainTxExecuted && (
            <div>
              <h3>Main Transaction</h3>
              <p>Please sign the main transaction using your wallet.</p>
              <Button onClick={signAndExecuteMainTransaction} disabled={loading}>
                Sign and Execute Main Transaction
              </Button>
            </div>
          )}
        </div>
      )}

      {step === 'completed' && (
        <div>
          <h2>Transaction Executed Successfully</h2>
          <p>Your transaction has been processed.</p>
          <pre>{JSON.stringify(executedTx, null, 2)}</pre>
          <Button onClick={() => setStep('tokenSelection')} disabled={loading}>
            Start Over
          </Button>
        </div>
      )}
    </div>
  );
}

const Button = (props: React.ButtonHTMLAttributes<HTMLButtonElement>) => (
  <button
    className="bg-indigo-600 text-sm font-medium text-white rounded-lg px-4 py-3 disabled:cursor-not-allowed disabled:opacity-60"
    {...props}
  />
);
