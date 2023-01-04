## casper-cep78-js-client

This package was created to help JS/TS users with the [cep-78-enhanced-nft](https://github.com/casper-ecosystem/cep-78-enhanced-nft) and is published in `npm` as the [casper-cep78-js-client](https://www.npmjs.com/package/casper-cep78-js-client). It was built on top of the [casper-js-sdk](https://github.com/casper-ecosystem/casper-js-sdk).

Users can treat this package as a deploy builder for all of these possible interactions:

- contract installation including different configurations
- token minting
- token transfers
- token burning
- approvals
- changing configurations after installation
- setting token metadata
- storing some of the contract-related data in an Account's `NamedKeys`

## Usage

1. To install, run:

   `npm i -S casper-cep78-js-client`

2. Import the contract in your code:

   `import { CEP78Client } from 'casper-cep78-js-client'`

3. If you want to install it, look at the `install` method and all of the possible configuration options (`InstallArgs`).

4. If you want to start working with a previously installed contract, use the `setContractHash(contractHash)` method.


## Examples 

As a good starting point, you can look into the `examples/` directory to see the most common usage scenarios (`install.ts` and `usage.ts`). You can install the contract and run the example scenario by running the `npm run example:install` and `npm run example:usage`. You will need to specify environment variables, preferebly in a `client-js/.env` file. Here are some example values for the environment variables that need to be specified: 

```
NODE_URL=http://localhost:11101/rpc
NETWORK_NAME=casper-net-1
MASTER_KEY_PAIR_PATH=/Users/someuser/.casper/casper-node/utils/nctl/assets/net-1/faucet
USER1_KEY_PAIR_PATH=/Users/someuser/.casper/casper-node/utils/nctl/assets/net-1/users/user-1
```
