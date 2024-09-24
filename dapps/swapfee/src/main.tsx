// src/main.tsx

import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import { IotaClientProvider, createNetworkConfig, WalletProvider } from '@iota/dapp-kit';
import { IotaClientOptions } from '@iota/iota-sdk/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import './styles/App.css';

// Configure the Hackanet network
const { networkConfig } = createNetworkConfig({
  hackanet: {
    url: 'https://api.hackanet.iota.cafe/',
    variables: {
      faucetUrl: 'https://faucet.hackanet.iota.cafe/gas',
    },
  },
});

// Create a QueryClient instance for react-query
const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <IotaClientProvider networks={networkConfig} defaultNetwork="hackanet">
      <WalletProvider enableUnsafeBurner>
          <App />
        </WalletProvider>
      </IotaClientProvider>
    </QueryClientProvider>
  </React.StrictMode>
);
