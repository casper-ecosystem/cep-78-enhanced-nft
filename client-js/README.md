## casper-cep78-js-client

This package was created to help JS/TS users dealing with [cep-78-enhanced-nft](https://github.com/casper-ecosystem/cep-78-enhanced-nft). It was built on top of [casper-js-sdk](https://github.com/casper-ecosystem/casper-js-sdk)

It can be treated as a deploy builder for all of the possible interactions:

- contract installation including different configurations
- token minting
- token transfers
- token burning
- approvals
- changing configuration after installation
- setting token metadata
- storing some of the contract related data in Account's `NamedKeys`

## Usage

1. To install, run `npm i -S capser-cep78-js-client`. 
2. Import the contract in your code `import { CEP78Client } from 'capser-cep78-js-client'`
3. If you want to install it, look at the `install` method and all of the possible configuration options (`InstallArgs`).
4. If you want to start working with a previously installed contract, use the `setContractHash(contractHash)` method.


## Examples 

As a good starting point, you can also look into the `examples/` directory to see the most common usage scenarios (`install.ts` and `usage.ts`). You can install the contract and run the example scenario by running the `npm run example:install` and `npm run example:usage`. You will just need to provide following env variables (preferebly in a `client-js/.env`).

```
NODE_URL=http://localhost:11101/rpc
NETWORK_NAME=casper-net-1
MASTER_KEY_PAIR_PATH=/Users/someuser/.casper/casper-node/utils/nctl/assets/net-1/faucet
USER1_KEY_PAIR_PATH=/Users/someuser/.casper/casper-node/utils/nctl/assets/net-1/users/user-1
```
