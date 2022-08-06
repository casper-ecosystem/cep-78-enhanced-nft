import {
  CasperClient,
  Contracts,
  CLPublicKey,
  CLKey,
  CLAccountHash,
  CLByteArray,
  CLValueBuilder,
  RuntimeArgs,
  Keys,
  decodeBase64,
} from "casper-js-sdk";

import {
  CEP78Client,
  NFTOwnershipMode,
  NFTKind,
  NFTMetadataKind,
  NFTIdentifierMode,
  MetadataMutability,
} from "../src/index";

import {
  KEYS,
  getBinary,
  getDeploy,
  getAccountInfo,
  getAccountNamedKeyValue,
} from "./common";

const install = async () => {
  const cc = new CEP78Client(process.env.NODE_URL!, process.env.NETWORK_NAME!);

  const installDeploy = await cc.install(
    getBinary("../contract/target/wasm32-unknown-unknown/release/contract.wasm"),
    {
      collectionName: "my-collection",
      collectionSymbol: "CEP",
      totalTokenSupply: "1000",
      ownershipMode: NFTOwnershipMode.Minter,
      nftKind: NFTKind.Physical,
      jsonSchema: {
        properties: {
          firstName: { name: "first name", description: "", required: true },
        },
      },
      nftMetadataKind: NFTMetadataKind.CustomValidated,
      identifierMode: NFTIdentifierMode.Ordinal,
      metadataMutability: MetadataMutability.Immutable,
    },
    "200000000000",
    KEYS.publicKey,
    [KEYS]
  );

  const hash = await installDeploy.send(process.env.NODE_URL!);

  console.log(`... Contract installation deployHash: ${hash}`);

  await getDeploy(process.env.NODE_URL!, hash);

  console.log(`... Contract installed successfully.`);

  let accountInfo = await getAccountInfo(
    process.env.NODE_URL!,
    KEYS.publicKey
  );

  console.log(`... Account Info: `);
  console.log(JSON.stringify(accountInfo, null, 2));

  const contractHash = await getAccountNamedKeyValue(
    accountInfo,
    `nft_contract`
  );

  const contractPackageHash = await getAccountNamedKeyValue(
    accountInfo,
    `nft_contract_package`
  );

  console.log(`... Contract Hash: ${contractHash}`);
  console.log(`... Contract Package Hash: ${contractPackageHash}`);
};

install();
