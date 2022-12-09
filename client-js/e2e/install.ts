import {
  CEP78Client,
  NFTOwnershipMode,
  NFTKind,
  NFTMetadataKind,
  NFTIdentifierMode,
  MetadataMutability,
  MintingMode,
} from "../src/index";

import {
  FAUCET_KEYS,
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
          registrationDate: {
            name: "registrationDate",
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
      mintingMode: MintingMode.Installer,
    },
    "250000000000",
    FAUCET_KEYS.publicKey,
    [FAUCET_KEYS]
  );

  const hash = await installDeploy.send(process.env.NODE_URL!);

  console.log(`... Contract installation deployHash: ${hash}`);

  await getDeploy(process.env.NODE_URL!, hash);

  console.log(`... Contract installed successfully.`);

  const accountInfo = await getAccountInfo(
    process.env.NODE_URL!,
    FAUCET_KEYS.publicKey
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
