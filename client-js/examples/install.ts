import {
  CEP78Client,
  NFTOwnershipMode,
  NFTKind,
  NFTMetadataKind,
  NFTIdentifierMode,
  MetadataMutability,
  OwnerReverseLookupMode,
  MintingMode,
  EventsMode
} from "../src/index";

import {
  FAUCET_KEYS,
  getDeploy,
  getAccountInfo,
  getAccountNamedKeyValue,
} from "./common";

const install = async () => {
  const cc = new CEP78Client(process.env.NODE_URL!, process.env.NETWORK_NAME!);

  const collectionName = "my-collection";

  const installDeploy = await cc.install(
    {
      collectionName,
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
      ownerReverseLookupMode: OwnerReverseLookupMode.Complete,
      eventsMode: EventsMode.CES
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
    `cep78_contract_hash_${collectionName}`
  );

  const contractPackageHash = await getAccountNamedKeyValue(
    accountInfo,
    `cep78_contract_package_${collectionName}`
  );

  console.log(`... Contract Hash: ${contractHash}`);
  console.log(`... Contract Package Hash: ${contractPackageHash}`);
};

install();
