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

## Commands

### Run the parser

```bash 
npm run parser <path/to/your/contract.move>
```

### Run the test

```bash 
npm run tests
```