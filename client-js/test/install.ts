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
  DeployUtil
} from "casper-js-sdk";

import {
  CEP78Client,
  NFTOwnershipMode,
  NFTKind,
  NFTMetadataKind,
  NFTIdentifierMode,
  MetadataMutability,
  MintingMode
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
    {
      collectionName: "my-collection",
      collectionSymbol: "AMAG-ASSETS",
      totalTokenSupply: "10000",
      ownershipMode: NFTOwnershipMode.Transferable,
      nftKind: NFTKind.Physical,
      jsonSchema: {
        properties: {
          type: { name: "type", description: "", required: true },
          make: { name: "make", description: "", required: true },
          model: { name: "model", description: "", required: true },
          fuelType: { name: "fuelType", description: "", required: false },
          engineCapacity: {
            name: "engineCapacity",
            description: "",
            required: false,
          },
          vin: { name: "vin", description: "", required: true },
          registerationDate: {
            name: "registerationDate",
            description: "",
            required: true,
          },
          typeCertificate: {
            name: "typeCertificate",
            description: "",
            required: false,
          },
        },
      },
      nftMetadataKind: NFTMetadataKind.CustomValidated,
      identifierMode: NFTIdentifierMode.Ordinal,
      metadataMutability: MetadataMutability.Immutable,
      mintingMode: MintingMode.Installer
    },
    "165000000000",
    KEYS.publicKey,
    [KEYS]
  );

  // console.log(JSON.stringify(DeployUtil.deployToJson(installDeploy), null, 2));

  const hash = await installDeploy.send(process.env.NODE_URL!);

  console.log(`... Contract installation deployHash: ${hash}`);

  await getDeploy(process.env.NODE_URL!, hash);

  console.log(`... Contract installed successfully.`);

  let accountInfo = await getAccountInfo(process.env.NODE_URL!, KEYS.publicKey);

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
