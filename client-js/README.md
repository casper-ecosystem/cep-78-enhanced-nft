# CEP-78 JavaScript Client Tutorial

This tutorial outlines usage of the JavaScript client available for the CEP-78 Enhanced NFT Standard.

Further information on the CEP-78 Enhanced NFT Standard itself can be found [here](https://github.com/casper-ecosystem/cep-78-enhanced-nft).

## Installing Dependencies

Dependencies can be installed using the following command in the `client-js` directory:

```js
npm i
```

## Installing a CEP-78 Contract using the JavaScript Client

The `install` method crafts a [Deploy](https://docs.casperlabs.io/design/casper-design/#execution-semantics-deploys) using `InstallArgs` and sends that Deploy to the specified Casper network.

```js

const install = async () => {
  const cc = new CEP78Client(process.env.NODE_URL!, process.env.NETWORK_NAME!);

  const installDeploy = await cc.install(
    {
      collectionName: "my-collection",
      collectionSymbol: "MY-NFTS",
      totalTokenSupply: "1000",
      ownershipMode: NFTOwnershipMode.Transferable,
      nftKind: NFTKind.Physical,
      jsonSchema: {
        properties: {
          color: { name: "color", description: "", required: true },
          size: { name: "size", description: "", required: true },
          material: { name: "material", description: "", required: true },
          condition: { name: "condition", description: "", required: false },
        },
      },
      nftMetadataKind: NFTMetadataKind.CustomValidated,
      identifierMode: NFTIdentifierMode.Ordinal,
      metadataMutability: MetadataMutability.Immutable,
      mintingMode: MintingMode.Installer,
    },
    "165000000000",
    FAUCET_KEYS.publicKey,
    [FAUCET_KEYS]
  );

  const hash = await installDeploy.send(process.env.NODE_URL!);

```

`InstallArgs` are specified in the associated `.env` file as follows:

* `collectionName` - The name of the NFT collection, passed in as a `String`. This parameter is required and cannot be changed post installation.

* `collectionSymbol` - The symbol representing a given NFT collection, passed in as a `String`. This parameter is required and cannot be changed post installation.

* `totalTokenSupply` - The total number of NFTs that a specific instance of a contract will mint passed in as a `U64` value. This parameter is required and cannot be changed post installation.

* `ownershipMode` - The `OwnershipMode` modality that dictates the ownership behavior of the NFT contract. This argument is passed in as a `u8` value and is required at the time of installation.

* `nftKind` - The `NFTKind` modality that specifies the off-chain items represented by the on-chain NFT data. This argument is passed in as a `u8` value and is required at the time of installation.

* `jsonSchema` - The JSON schema for the NFT tokens that will be minted by the NFT contract passed in as a `String`. This parameter is required if the metadata kind is set to `CustomValidated(4)` and cannot be changed post installation.

* `nftMetadataKind` - The metadata schema for the NFTs to be minted by the NFT contract. This argument is passed in as a `u8` value and is required at the time of installation.

* `identifierMode` - The `NFTIdentifierMode` modality dictates the primary identifier for NFTs minted by the contract. This argument is passed in as a `u8` value and is required at the time of installation.

* `metadataMutability` - The `MetadataMutability` modality dictates whether the metadata of minted NFTs can be updated. This argument is passed in as a `u8` value and is required at the time of installation.

* `mintingmode?` - The `MintingMode` modality that dictates the access to the `mint()` entry-point in the NFT contract. This is an optional parameter that will default to restricting access to the installer of the contract. This parameter cannot be changed once the contract has been installed.

* `holdermode?` - The `NFTHolderMode` modality dictates which entities can hold NFTs. This is an optional parameter and will default to a mixed mode allowing either `Accounts` or `Contracts` to hold NFTs. This parameter cannot be changed once the contract has been installed.

* `burnMode?` - The `BurnMode` modality dictates whether minted NFTs can be burnt. This is an optional parameter and will allow tokens to be burnt by default. This parameter cannot be changed once the contract has been installed.

Further information on CEP-78 modality options can be found in the base [cep-78-enhanced-nft](https://github.com/ACStoneCL/cep-78-enhanced-nft) repository on GitHub.

## Minting a Token

The CEP-78 JS Client includes code to construct a deploy that will `Mint` a token, as follows:

```js

  const mintDeploy = await cc.mint(
    {
      owner: FAUCET_KEYS.publicKey,
      meta: {
        color: "Blue",
        size: "Medium",
        material: "Aluminum",
        condition: "Used",
      },
    },
    "1000000000",
    FAUCET_KEYS.publicKey,
    [FAUCET_KEYS]
  );

  const mintDeployHash = await mintDeploy.send(NODE_URL!);

```
The arguments adhere to those provided in the original installation.

## Transferring a Token

After minting one or more tokens, you can then use the following code to transfer the tokens between accounts:

```js

  const transferDeploy = await cc.transfer(
    {
      tokenId: "0",
      source: FAUCET_KEYS.publicKey,
      target: USER1_KEYS.publicKey,
    },
    "13000000000",
    FAUCET_KEYS.publicKey,
    [FAUCET_KEYS]
  );

  const transferDeployHash = await transferDeploy.send(NODE_URL!);

```

Transferring accepts the following arguments:

* `tokenId` - The sequential ID assigned to a token in mint order.

* `source` - The account sending the token in question.

* `target` - The account receiving the transferred token.

## Burning a Token

The following code shows how to burn a minted NFT that you hold and have access rights to, requiring only the `tokenId` argument:

```js

  const burnDeploy = await cc.burn(
    { tokenId: "0" },
    "13000000000",
    USER1_KEYS.publicKey,
    [USER1_KEYS]
  );

  const burnDeployHash = await burnDeploy.send(NODE_URL!);

```

## Testing

### Running an Install Test

This repository includes a test script for installing a CEP-78 contract instance.

You will need to define the following variables in the `.env` file:

* `NODE_URL` - The address of a node. If you are testing using [NCTL](https://docs.casperlabs.io/dapp-dev-guide/building-dapps/setup-nctl/), this will be `http://localhost:11101/rpc`.

* `NETWORK_NAME` - The name of the Casper network you are testing on, `casper-net-1` when testing using a local network with NCTL.

* `MASTER_KEY_PAIR_PATH` - The path to the key pair of the minting account.

* `USER1_KEY_PAIR_PATH` - The path to an additional account's key pair for use in testing transfer features.

You may also need to install associated dependencies using:

```js
npm i
```

This test can be run using the following command:

```js
npm run test:install
```

The test will then return the installation's `deployHash`, and inform you when the installation is successful.

The test will then provide the installing account's information, which will include the CEP-78 NFT contract's hash and package hash.


### Running a Usage Test

A usage test uses the same variables as the Install test above, but tests the functionality of the contract after installation.

The usage test can be run using the following command:

```js
npm run test:usage
```

This test will acquire the contract's hash and package hash, prior to sending three separate deploys to perform several function tests as follows:

* `Mint` - The test will attempt to mint an NFT using the installation account.

* `Transfer` - The test will transfer the previously minted NFT to a second account (USER1 as defined in the variables.)

* `Burn` - The test will burn the minted NFT.
