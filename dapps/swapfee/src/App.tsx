// src/App.tsx

import React, { useState } from 'react';
import { ConnectButton, useCurrentAccount, useSignTransactionBlock, useIotaClient } from '@iota/dapp-kit';
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
  const currentAccount = useCurrentAccount();
  const { mutateAsync: signTransactionBlock } = useSignTransactionBlock();
  const [loading, setLoading] = useState(false);

  // State variables
  const [userAddress, setUserAddress] = useState('');
  const [userTokens, setUserTokens] = useState<Token[]>([]);
  const [selectedToken, setSelectedToken] = useState<Token | null>(null);
  const [tokenAmount, setTokenAmount] = useState<bigint>(BigInt(0));
  const [gasPaymentTx, setGasPaymentTx] = useState<any>(null);
  const [gasPaymentExecuted, setGasPaymentExecuted] = useState(false);
  const [mainTx, setMainTx] = useState<any>(null);
  const [mainTxExecuted, setMainTxExecuted] = useState(false);
  const [executedTx, setExecutedTx] = useState<any>(null);

  // Step management
  const [step, setStep] = useState<
    'addressInput' | 'tokenSelection' | 'mainTransaction' | 'review' | 'transaction' | 'completed'
  >('addressInput');

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

  // Fetch user's tokens when they provide an address
  const fetchTokens = async () => {
    setLoading(true);
    try {
      const tokens = await fetchUserTokens(userAddress);
      setUserTokens(tokens);
      if (tokens.length > 0) {
        setStep('tokenSelection');
      } else {
        alert('No tokens found at the provided address.');
      }
    } catch (error) {
      console.error('Error fetching user tokens:', error);
      alert('Failed to fetch tokens. Please check the address and try again.');
    } finally {
      setLoading(false);
    }
  };

  // Handle token selection for gas payment
  const handleTokenSelect = (token: Token) => {
    setLoading(true);
    try {
      const amount = calculateDynamicGasFee(token);
      setSelectedToken(token);
      setTokenAmount(amount);
      setStep('mainTransaction');
    } catch (error: any) {
      console.error('Error calculating gas fee:', error);
      alert(error.message);
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

  // const ourAddress = '0x4dafe69422fccffdd3804d76077f55060002c8e4a4b156b5cd79712b9cf92a4b' //'YOUR_IOTA_ADDRESS'; // Replace with your actual address

  // Handle gas payment transaction
// src/App.tsx

const handleGasPayment = async () => {
  if (!userAddress || !selectedToken) return;
  setLoading(true);
  try {
    const tx = new TransactionBlock();
    const ourAddress = '0x4dafe69422fccffdd3804d76077f55060002c8e4a4b156b5cd79712b9cf92a4b'; // Replace with your actual address

    const gasFeeAmount = tokenAmount; // Already a BigInt
    const coinBalance = selectedToken.balance;

    if (coinBalance < gasFeeAmount) {
      throw new Error('Insufficient balance to cover the gas fee.');
    }

    const coinObject = tx.object(selectedToken.coinId);

    // Split the coin into gas fee and remaining
    const splitResult = tx.splitCoins(coinObject, [
      tx.pure(gasFeeAmount.toString()),
    ]);

    console.log('splitResult:', splitResult); // Log the split results

    if (splitResult.length === 0) {
      throw new Error('No coins returned from splitCoins.');
    }

    // Transfer the first split coin to our address (gas payment)
    const gasPaymentCoin = splitResult[0];
    tx.transferObjects([gasPaymentCoin], tx.pure(ourAddress));

    // Transfer any remaining coins back to the user
    for (let i = 1; i < splitResult.length; i++) {
      const remainingCoin = splitResult[i];
      tx.transferObjects([remainingCoin], tx.pure(userAddress));
    }

    const gasPaymentBytes = await tx.build({ client, onlyTransactionKind: true });

    const sponsoredGasPaymentTx = await sponsorTransaction(
      userAddress,
      gasPaymentBytes,
      true // isGasPayment
    );

    setGasPaymentTx(sponsoredGasPaymentTx);
    setStep('transaction');
  } catch (error: any) {
    console.error('Error during gas payment:', error);
    alert(`Failed to create gas payment transaction: ${error.message}`);
  } finally {
    setLoading(false);
  }
};

  
  

  // Sign and execute gas payment transaction
  const signAndExecuteGasPayment = async () => {
    if (!gasPaymentTx) return;
    setLoading(true);
    try {
      const signedGasPaymentTx = await signTransactionBlock({
        transactionBlock: TransactionBlock.from(gasPaymentTx.bytes),
      });

      const executedGasPayment = await client.executeTransactionBlock({
        transactionBlock: signedGasPaymentTx.transactionBlockBytes,
        signature: [signedGasPaymentTx.signature, gasPaymentTx.signature],
        options: {
          showEffects: true,
          showEvents: true,
          showObjectChanges: true,
        },
      });

      if (executedGasPayment.effects?.status?.status === 'success') {
        setGasPaymentExecuted(true);
        // Proceed to create main transaction
        await createMainTransaction();
      } else {
        console.error('Gas payment transaction failed:', executedGasPayment.effects?.status);
        alert('Gas payment transaction failed.');
      }
    } catch (error) {
      console.error('Error executing gas payment transaction:', error);
      alert('Failed to execute gas payment transaction.');
    } finally {
      setLoading(false);
    }
  };

  // Create main transaction
  const createMainTransaction = async () => {
    setLoading(true);
    try {
      const tx = new TransactionBlock();

      // Transfer the specified amount of the token to the recipient
      const tokenToSend = userTokens.find(
        (token) => token.coinId === mainTxDetails.tokenToSend?.coinId
      );

      if (!tokenToSend) {
        alert('Selected token not found in your wallet.');
        return;
      }

      // Use the amount to send (integer)
      const amountToSend = BigInt(mainTxDetails.amountToSend);

      const coinObject = tx.object(tokenToSend.coinId);

      // Split the coin to get the amount to send
      const [amountToSendCoin, remainingCoin] = tx.splitCoins(coinObject, [
        tx.pure(amountToSend.toString()),
      ]);

      // Transfer the amount to the recipient
      tx.transferObjects([amountToSendCoin], tx.pure(mainTxDetails.recipientAddress));

      // Transfer the remaining coin back to the user
      if (remainingCoin) {
        tx.transferObjects([remainingCoin], tx.pure(userAddress));
      }

      const mainTxBytes = await tx.build({ client, onlyTransactionKind: true });
      const sponsoredMainTx = await sponsorTransaction(
        userAddress,
        mainTxBytes,
        false // isGasPayment
      );

      setMainTx(sponsoredMainTx);
    } catch (error) {
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
    if (step === 'tokenSelection') {
      setStep('addressInput');
    } else if (step === 'mainTransaction') {
      setStep('tokenSelection');
    } else if (step === 'review') {
      setStep('mainTransaction');
    } else if (step === 'transaction') {
      setStep('review');
    }
  };

  // Render component
  return (
    <div className="p-8">
      {step === 'addressInput' && (
        <div>
          <h2>Enter Your IOTA Address</h2>
          <input
            type="text"
            value={userAddress}
            onChange={(e) => setUserAddress(e.target.value)}
            placeholder="Enter your IOTA address"
            className="border p-2 w-full mb-4"
          />
          <Button onClick={fetchTokens} disabled={loading || !userAddress}>
            Fetch Tokens
          </Button>
          {loading && <p>Loading...</p>}
        </div>
      )}

      {step === 'tokenSelection' && (
        <div>
          <h2>Select a Token to Pay for Gas Fees</h2>
          {loading && <p>Loading...</p>}
          {userTokens.length === 0 && <p>No tokens found at the provided address.</p>}
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
          <h3>Gas Payment</h3>
          <p>
            You need to send{' '}
            <strong>
              {tokenAmount.toString()} {getTokenName(selectedToken?.coinType || '')}
            </strong>{' '}
            to cover the gas fees.
          </p>
          <h3>Main Transaction</h3>
          <p>
            You will send{' '}
            <strong>
              {mainTxDetails.amountToSend} {getTokenName(mainTxDetails.tokenToSend?.coinType || '')}
            </strong>{' '}
            to <strong>{mainTxDetails.recipientAddress}</strong>.
          </p>
          <Button onClick={handleGasPayment} disabled={loading}>
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
          {!gasPaymentExecuted && (
            <div>
              <h3>Gas Payment Transaction</h3>
              <p>Please sign the gas payment transaction using your wallet.</p>
              <Button onClick={signAndExecuteGasPayment} disabled={loading}>
                Sign and Execute Gas Payment Transaction
              </Button>
            </div>
          )}
          {gasPaymentExecuted && !mainTxExecuted && (
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
          <Button onClick={() => setStep('addressInput')} disabled={loading}>
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
