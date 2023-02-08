import { CEP78Client, OwnerReverseLookupMode } from "../src/index";

import {
  FAUCET_KEYS,
  USER1_KEYS,
  USER2_KEYS,
  getDeploy,
  getAccountInfo,
  getAccountNamedKeyValue,
  printHeader,
} from "./common";

import { DeployUtil, CLPublicKey, EventStream, EventName, CLValueParsers, CLTypeTag, CLMap, CLValue, CLValueBuilder } from "casper-js-sdk";

const { NODE_URL, EVENT_STREAM_ADDRESS } = process.env;

const runDeployFlow = async (deploy: DeployUtil.Deploy) => {
  const deployHash = await deploy.send(NODE_URL!);

  console.log("...... Deploy hash: ", deployHash);
  console.log("...... Waiting for the deploy...");

  await getDeploy(NODE_URL!, deployHash);

  console.log(`...... Deploy ${deployHash} succedeed`);
};

enum CEP47Events {
  MintOne = "cep47_mint_one",
  TransferToken = "cep47_transfer_token",
  BurnOne = "cep47_burn_one",
  MetadataUpdate = 'cep47_metadata_update',
  ApproveToken = 'cep47_approve_token'
}

const CEP47EventParser = (
  {
    contractPackageHash,
    eventNames,
  }: { contractPackageHash: string; eventNames: CEP47Events[] },
  value: any
) => {
  if (value.body.DeployProcessed.execution_result.Success) {
    const { transforms } =
      value.body.DeployProcessed.execution_result.Success.effect;

        const cep47Events = transforms.reduce((acc: any, val: any) => {
          if (
            val.transform.hasOwnProperty("WriteCLValue") &&
            val.transform.WriteCLValue.cl_type === 'Any'
            // typeof val.transform.WriteCLValue.parsed === "object" &&
            // val.transform.WriteCLValue.parsed !== null
          ) {
            const maybeCLValue = CLValueParsers.fromBytesWithType(
              Buffer.from(val.transform.WriteCLValue.bytes, 'hex')
            );
            const clValue = maybeCLValue.unwrap();

            console.log(val);
            console.log('value',clValue.toJSON());

            if (clValue && clValue.clType().tag === CLTypeTag.Map) {
              const hash = (clValue as CLMap<CLValue, CLValue>).get(
                CLValueBuilder.string("contract_package_hash")
              );

              console.log(hash);

              const event = (clValue as CLMap<CLValue, CLValue>).get(CLValueBuilder.string("event_type"));

              console.log(event);

              if (
                hash &&
                // NOTE: Calling toLowerCase() because current JS-SDK doesn't support checksumed hashes and returns all lower case value
                // Remove it after updating SDK
                hash.value() === contractPackageHash.slice(5).toLowerCase() &&
                event &&
                eventNames.includes(event.value())
              ) {
                acc = [...acc, { name: event.value(), clValue }];
              }
            }
          }
          return acc;
        }, []);

    return { error: null, success: !!cep47Events.length, data: cep47Events };
  }

  return null;
};

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
    `cep78_contract_hash_my-collection`
  );

  const contractPackageHash = await getAccountNamedKeyValue(
    accountInfo,
    `cep78_contract_package_my-collection`
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

  const ownerReverseLookupModeSetting = await cc.getReportingModeConfig();
  console.log(
    `OwnerReverseLookupMode: ${ownerReverseLookupModeSetting}`
  );

  const useSessionCode =
    ownerReverseLookupModeSetting ===
    OwnerReverseLookupMode[OwnerReverseLookupMode.Complete];

  const JSONSetting = await cc.getJSONSchemaConfig();

  const es = new EventStream(EVENT_STREAM_ADDRESS!);
  es.subscribe(EventName.DeployProcessed, (event) => {
    const parsedEvents = CEP47EventParser({
      contractPackageHash, 
      eventNames: [
        CEP47Events.MintOne,
        CEP47Events.TransferToken,
        CEP47Events.BurnOne,
        CEP47Events.MetadataUpdate,
        CEP47Events.ApproveToken
      ]
    }, event);

    if (parsedEvents && parsedEvents.success) {
      console.log("*** EVENT ***");
      console.log(parsedEvents.data);
      console.log("*** ***");
    }
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
  accountInfo = await getAccountInfo(NODE_URL!, FAUCET_KEYS.publicKey);

  const storedOwnerValue = await getAccountNamedKeyValue(
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

run();
