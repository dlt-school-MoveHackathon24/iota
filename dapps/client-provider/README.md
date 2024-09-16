# Client provider

## Get started

### Clone the repository branch

```bash
git clone -b movers/client-provider git@github.com:temp-dlt-school-24-org/iota.git
```

### Build and link the TypeScript SDK

From the root `iota` folder:

```bash 
pnpm sdk build
pnpm link ../../sdk/typescript
```

From the `client-provider` folder: 

```bash 
pnpm link ../../sdk/typescript
```

### Create a `.env` file 

```bash 
touch .env
```

This file must define the passphrase for the Ed25519 private key, the RPC url for interacting with the chain and the faucet url.

#### Example

```bash
#.env

IOTA_CP_PASSPHRASE=insert your passphrase here

IOTA_CP_RPC=https://api.hackanet.iota.cafe/
IOTA_CP_FAUCET=https://faucet.hackanet.iota.cafe/gas
```

## Commands

### Run the parser module

```bash 
npm run parse-module <path/to/your/contract.move>
```

#### Example

```bash 
npm run parse-module src/modules/luckynumber.move
```

### Execute the main script to test the client provider

```bash 
npm run tests
```