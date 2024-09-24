# SwapFee DApp

The SwapFee DApp allows users to perform transactions or call smart contract functions on the IOTA network without holding native IOTA tokens for gas fees. Instead, users can pay gas fees using custom tokens they already possess. The service sponsors the gas fees in exchange for receiving custom tokens from the user.

## Features

- Allows users to pay gas fees using custom tokens.
- Users can perform any transaction, such as transferring tokens to another address.
- The service sponsors the gas fees after verifying gas payment.

## Prerequisites

- Node.js >= 20.0.0
- pnpm >= 9.0.0

## Installation

1. **Clone the repository:**

   ```bash
   git clone https://your-repo-url.git


STRUCTURE: 
swapfee/
├── package.json
├── tsconfig.json
├── .eslintrc.json
├── .prettierrc
├── public/
│   └── index.html
├── src/
│   ├── App.tsx
│   ├── index.tsx
│   ├── utils/
│   │   ├── fetchUserTokens.ts
│   │   ├── calculateDynamicGasFee.ts
│   │   ├── SponsorTransaction.ts
│   │   └── rpc.ts
│   └── styles/
│       └── App.css
├── README.md
└── (Other configuration files as needed)
