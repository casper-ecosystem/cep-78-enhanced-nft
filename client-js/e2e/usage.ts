import { CEP78Client } from "../src/index";

import {
  FAUCET_KEYS,
  USER1_KEYS,
  USER2_KEYS,
  getDeploy,
  getAccountInfo,
  getAccountNamedKeyValue,
  printHeader,
} from "./common";

import { DeployUtil, CLPublicKey } from "casper-js-sdk";

const { NODE_URL } = process.env;

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

  let accountInfo = await getAccountInfo(NODE_URL!, FAUCET_KEYS.publicKey);

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

  await cc.setContractHash(contractHash, undefined);

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
  printHeader("Register");

  const registerDeploy = await cc.register(
    {
      tokenOwner: FAUCET_KEYS.publicKey
    },
    "1000000000",
    FAUCET_KEYS.publicKey,
    [FAUCET_KEYS]
  );

  const registerDeployHash = await registerDeploy.send(NODE_URL!);

  console.log("...... Deploy hash: ", registerDeployHash);
  console.log("...... Waiting for the deploy...");

  await getDeploy(NODE_URL!, registerDeployHash);

  console.log("Deploy Succedeed");

  /* Mint */
  printHeader("Mint");

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

  console.log("...... Deploy hash: ", mintDeployHash);
  console.log("...... Waiting for the deploy...");

  await getDeploy(NODE_URL!, mintDeployHash);

  console.log("Deploy Succedeed");

  /* Token details */

  printTokenDetails("0", FAUCET_KEYS.publicKey);

  /* Transfer */
  printHeader("Transfer");

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

  console.log("...... Deploy hash: ", transferDeployHash);
  console.log("...... Waiting for the deploy...");

  await getDeploy(NODE_URL!, transferDeployHash);

  console.log("Deploy Succedeed");

  /* Token details */

  printTokenDetails("0", USER1_KEYS.publicKey);

  /* Store owner of at account named key */
  printHeader("Store owner of");

  const storeOwnerOfDeploy = cc.storeOwnerOf(
    {
      keyName: "stored_owner_of_token",
      tokenId: "0",
    },
    "13000000000",
    FAUCET_KEYS.publicKey,
    [FAUCET_KEYS]
  );

  const storeOwnerOfDeployHash = await storeOwnerOfDeploy.send(NODE_URL!);
  console.log("...... Deploy hash: ", storeOwnerOfDeployHash);
  console.log("...... Waiting for the deploy...");

  await getDeploy(NODE_URL!, storeOwnerOfDeployHash); 

  console.log("Deploy Succedeed");

  accountInfo = await getAccountInfo(NODE_URL!, FAUCET_KEYS.publicKey);

  console.log(`\n=====================================\n`);

  console.log(`... Account Info: `);
  console.log(JSON.stringify(accountInfo, null, 2));

  const storedOwnerValue = await getAccountNamedKeyValue(
    accountInfo,
    `stored_owner_of_token`
  );

  console.log('.. storedOwnerValue: ', storedOwnerValue);

  /* Burn */
  printHeader("Burn");

  const burnDeploy = await cc.burn(
    { tokenId: "0" },
    "13000000000",
    USER1_KEYS.publicKey,
    [USER1_KEYS]
  );

  const burnDeployHash = await burnDeploy.send(NODE_URL!);

  console.log("...... Deploy hash: ", burnDeployHash);
  console.log("...... Waiting for the deploy...");

  await getDeploy(NODE_URL!, burnDeployHash);

  console.log("Deploy Succedeed");
};

run();
