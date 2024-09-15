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

### Create a `passphrase.sk` file 

```bash 
touch passphrase.sk
```

Insert your passphrase (the words must be separated by spaces) in this file. This file won't be pushed to the repository as this behaviour is prohibited by the `.gitignore`.

## Commands

### Run the parser

```bash 
npm run parser <path/to/your/contract.move>
```

### Run the test

```bash 
npm run tests
```