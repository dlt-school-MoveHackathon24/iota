# Iota dApp Starter Template

This dApp was created using `@iota/create-dapp` that sets up a basic React
Client dApp using the following tools:

- [React](https://react.dev/) as the UI framework
- [TypeScript](https://www.typescriptlang.org/) for type checking
- [Vite](https://vitejs.dev/) for build tooling
- [Radix UI](https://www.radix-ui.com/) for pre-built UI components
- [ESLint](https://eslint.org/) for linting
- [`@iota/dapp-kit`]() for connecting to
  wallets and loading data
- [pnpm](https://pnpm.io/) for package management

## Deploying your Move code

### Install Iota cli

Before deploying your move code, ensure that you have installed the Iota CLI.
You can follow the
[Iota installation instruction](https://docs.iota.io/build/install) to get
everything set up.

This template uses `devnet` by default, so we'll need to set up a devnet
environment in the CLI:

```bash
iota client new-env --alias devnet --rpc https://fullnode.devnet.iota.io:443
iota client switch --env devnet
```

If you haven't set up an address in the iota client yet, you can use the
following command to get a new address:

```bash
iota client new-address secp256k1
```

This well generate a new address and recover phrase for you. You can mark a
newly created address as you active address by running the following command
with your new address:

```bash
iota client switch --address 0xYOUR_ADDRESS...
```

We can ensure we have some Iota in our new wallet by requesting Iota from the
faucet (make sure to replace the address with your address):

```bash
curl --location --request POST 'https://faucet.devnet.iota.io/gas' \
--header 'Content-Type: application/json' \
--data-raw '{
    "FixedAmountRequest": {
        "recipient": "<YOUR_ADDRESS>"
    }
}'
```

### Publishing the move package

The move code for this template is located in the `move` directory. To publish
it, you can enter the `move` directory, and publish it with the Iota CLI:

```bash
cd move
iota client publish --gas-budget 100000000 counter
```

In the output there will be an object with a `"packageId"` property. You'll want
to save that package ID to the `src/constants.ts` file as `PACKAGE_ID`:

```ts
export const DEVNET_COUNTER_PACKAGE_ID = "<YOUR_PACKAGE_ID>";
```

Now that we have published the move code, and update the package ID, we can
start the app.

## Starting your dApp

To install dependencies you can run

```bash
pnpm install
```

To start your dApp in development mode run

```bash
pnpm dev
```

## Building

To build your app for deployment you can run

```bash
pnpm build
```
