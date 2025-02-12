// eslint-disable-next-line eslint-comments/disable-enable-pair
/* eslint-disable no-console */
import {
  DeployUtil,
  CLPublicKey,
  EventStream,
  EventName,
  CasperServiceByJsonRPC
} from "casper-js-sdk";
import {
  CEP78Client,
  OwnerReverseLookupMode,
  CESEventParserFactory,
  EventItem,
} from "../src/index";

import {
  FAUCET_KEYS,
  USER1_KEYS,
  getDeploy,
  getAccountInfo,
  getAccountNamedKeyValue,
  printHeader,
} from "./common";


const { NODE_URL, EVENT_STREAM_ADDRESS } = process.env;

const runDeployFlow = async (deploy: DeployUtil.Deploy) => {
  const deployHash = await deploy.send(NODE_URL);

  console.log("...... Deploy hash: ", deployHash);
  console.log("...... Waiting for the deploy...");

  await getDeploy(NODE_URL, deployHash);

  console.log(`...... Deploy ${deployHash} succedeed`);
};

const usage = async () => {
  const cc = new CEP78Client(process.env.NODE_URL, process.env.NETWORK_NAME);

  const printTokenDetails = async (id: string, pk: CLPublicKey) => {
    const ownerOfToken = await cc.getOwnerOf(id);
    console.log(`> Owner of token ${id} is ${ownerOfToken}`);

    const ownerBalance = await cc.getBalanceOf(pk);
    console.log(`> Account ${pk.toAccountHashStr()} balance ${ownerBalance}`);

    const metadataOfZero = await cc.getMetadataOf(id);
    console.log(`> Token ${id} metadata`, metadataOfZero);
  };

  // eslint-disable-next-line @typescript-eslint/no-unsafe-call
  let accountInfo = await getAccountInfo(NODE_URL, FAUCET_KEYS.publicKey);

  console.log(`\n=====================================\n`);

  console.log(`... Account Info: `);
  console.log(JSON.stringify(accountInfo, null, 2));

  const contractHash = getAccountNamedKeyValue(
    accountInfo,
    `cep78_contract_hash_my-collection`
  );

  const contractPackageHash = getAccountNamedKeyValue(
    accountInfo,
    `cep78_contract_package_my-collection`
  );

  console.log(`... Contract Hash: ${contractHash}`);
  console.log(`... Contract Package Hash: ${contractPackageHash}`);

  cc.setContractHash(contractHash, undefined);

  console.log(`\n=====================================\n`);

  const allowMintingSetting = await cc.getAllowMintingConfig() as string;
  console.log(`AllowMintingSetting: ${allowMintingSetting}`);

  const burnModeSetting = await cc.getBurnModeConfig();
  console.log(`BurnModeSetting: ${burnModeSetting}`);

  const holderModeSetting = await cc.getHolderModeConfig();
  console.log(`HolderModeSetting: ${holderModeSetting}`);

  const identifierModeSetting = await cc.getIdentifierModeConfig();
  console.log(`IdentifierModeSetting: ${identifierModeSetting}`);

  const whitelistModeSetting = await cc.getWhitelistModeConfig();
  console.log(`WhitelistMode: ${whitelistModeSetting}`);

  const ownerReverseLookupModeSetting = await cc.getReportingModeConfig();
  console.log(`OwnerReverseLookupMode: ${ownerReverseLookupModeSetting}`);

  const useSessionCode =
    ownerReverseLookupModeSetting ===
    OwnerReverseLookupMode[OwnerReverseLookupMode.Complete];

  const casperClient = new CasperServiceByJsonRPC(NODE_URL);
  const cesEventParser = CESEventParserFactory({
    contractHashes: [contractHash],
    casperClient,
  });

  const es = new EventStream(EVENT_STREAM_ADDRESS);

  es.subscribe(EventName.DeployProcessed, (event: EventItem) => {
    cesEventParser(event)
      .then((parsedEvents) => {
        if (parsedEvents?.success) {
          console.log("*** EVENT ***");
          console.log(parsedEvents.data);
          console.log("*** ***");
        } else {
          console.log("*** EVENT NOT RELATED TO WATCHED CONTRACT ***");
        }
      })
      .catch((error) => {
        console.error("Error processing event:", error);
      });
  });

  es.start();

  /* Mint */
  printHeader("Mint");

  const mintDeploy = cc.mint(
    {
      owner: FAUCET_KEYS.publicKey,
      meta: {
        color: "Blue",
        size: "Medium",
        material: "Aluminum",
        condition: "Used",
      },
      collectionName: "my-collection",
    },
    { useSessionCode },
    "2000000000",
    FAUCET_KEYS.publicKey,
    [FAUCET_KEYS]
  );

  await runDeployFlow(mintDeploy);

  /* Token details */
  await printTokenDetails("0", FAUCET_KEYS.publicKey);

  if (useSessionCode) {
    /* Register */
    printHeader("Register");

    const registerDeployTwo = cc.register(
      {
        tokenOwner: USER1_KEYS.publicKey,
      },
      "1000000000",
      USER1_KEYS.publicKey,
      [USER1_KEYS]
    );

    await runDeployFlow(registerDeployTwo);
  }

  /* Transfer */
  printHeader("Transfer");

  const transferDeploy = cc.transfer(
    {
      tokenId: "0",
      source: FAUCET_KEYS.publicKey,
      target: USER1_KEYS.publicKey,
    },
    { useSessionCode },
    "13000000000",
    FAUCET_KEYS.publicKey,
    [FAUCET_KEYS]
  );

  await runDeployFlow(transferDeploy);

  /* Token details */
  await printTokenDetails("0", USER1_KEYS.publicKey);

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

  await runDeployFlow(storeOwnerOfDeploy);

  // Getting new account info to update namedKeys

  accountInfo = await getAccountInfo(NODE_URL, FAUCET_KEYS.publicKey);

  const storedOwnerValue = getAccountNamedKeyValue(
    accountInfo,
    `stored_owner_of_token`
  );

  console.log(".. storedOwnerValue UREF: ", storedOwnerValue);

  /* Burn */
  printHeader("Burn");

  const burnDeploy = cc.burn(
    { tokenId: "0" },
    "13000000000",
    USER1_KEYS.publicKey,
    [USER1_KEYS]
  );

  await runDeployFlow(burnDeploy);
};

usage()
  .then(() => {
    console.log("Usage completed successfully.");
  })
  .catch((error) => {
    console.error("Usage failed:", error);
  });