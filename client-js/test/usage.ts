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
  printHeader,
} from "./common";

import { DeployUtil, CLPublicKey } from "casper-js-sdk";

const { NODE_URL, NETWORK_NAME, CONTRACT_NAME } = process.env;

const run = async () => {
  const cc = new CEP78Client(process.env.NODE_URL!, process.env.NETWORK_NAME!);

  const printTokenDetails = async (id: string, pk: CLPublicKey) => {
    const ownerOfToken = await cc.getOwnerOf(id);
    console.log(`> Owner of token ${id} is ${ownerOfToken}`);

    const ownerBalance = await cc.getBalanceOf(pk);
    console.log(`> Account ${pk.toAccountHashStr()} balance ${ownerBalance}`);

    const metadataOfZero = await cc.getMetadataOf(id);
    console.log(`> Token ${id} metadata`, metadataOfZero);
  };

  let accountInfo = await getAccountInfo(NODE_URL!, KEYS.publicKey);

  console.log(`\n=====================================\n`);

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

  await cc.setContractHash(contractHash, undefined, true);

  console.log(`\n=====================================\n`);

  const allowMintingSetting = await cc.getAllowMintingConfig();
  console.log(`AllowMintingSetting: ${allowMintingSetting}`);

  const burnModeSetting = await cc.getBurnModeConfig();
  console.log(`BurnModeSetting: ${burnModeSetting}`);

  const holderModeSetting = await cc.getHolderModeConfig();
  console.log(`HolderModeSetting: ${holderModeSetting}`);

  const identifierModeSetting = await cc.getIdentifierModeConfig();
  console.log(`IdentifierModeSetting: ${identifierModeSetting}`);

  const whitelistModeSetting = await cc.getWhitelistModeConfig();
  console.log(`WhitelistMode: ${whitelistModeSetting}`);

  const JSONSetting = await cc.getJSONSchemaConfig();

  /* Mint */
  printHeader("Mint");

  const mintDeploy = await cc.mint(
    {
      owner: KEYS.publicKey,
      meta: {
        type: "vehicle",
        make: "Audi",
        model: "S3",
        fuelType: "petrol",
        engineCapacity: "2000",
        vin: "4Y1SL65848Z411439",
        registerationDate: "2019-10-01",
      },
    },
    "500000000000",
    KEYS.publicKey,
    [KEYS]
  );

  const mintDeployHash = await mintDeploy.send(NODE_URL!);

  console.log("...... Deploy hash: ", mintDeployHash);
  console.log("...... Waiting for the deploy...");

  await getDeploy(NODE_URL!, mintDeployHash);

  console.log("Deploy Succedeed");

  /* Token details */

  printTokenDetails("0", KEYS.publicKey);

  /* Transfer */
  printHeader("Transfer");

  const targetPK = CLPublicKey.fromHex(
    "011ee777efd4d3aaccef971393809c3d8e6facb9af4fd89daa707dad6c79b8477d"
  );

  const transferDeploy = await cc.transfer(
    {
      tokenId: "0",
      source: KEYS.publicKey,
      target: targetPK
    },
    "13000000000",
    KEYS.publicKey,
    [KEYS]
  );

  const transferDeployHash = await transferDeploy.send(NODE_URL!);

  console.log("...... Deploy hash: ", transferDeployHash);
  console.log("...... Waiting for the deploy...");

  await getDeploy(NODE_URL!, transferDeployHash);

  console.log("Deploy Succedeed");

  /* Token details */

  printTokenDetails("0", targetPK);

  /* Burn */
  printHeader("Burn");

  const burnDeploy = await cc.burn(
    { tokenId: "0" },
    "13000000000",
    KEYS.publicKey,
    [KEYS]
  );

  const burnDeployHash = await burnDeploy.send(NODE_URL!);

  console.log("...... Deploy hash: ", burnDeployHash);
  console.log("...... Waiting for the deploy...");

  await getDeploy(NODE_URL!, burnDeployHash);

  console.log("Deploy Succedeed");
};

run();