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