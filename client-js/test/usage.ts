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
  getDeploy,
  getAccountInfo,
  getAccountNamedKeyValue,
  printHeader,
} from "./common";

const { NODE_URL, NETWORK_NAME, CONTRACT_NAME } = process.env;

const run = async () => {
  const cc = new CEP78Client(
    process.env.NODE_URL!,
    process.env.NETWORK_NAME!
  );

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

  // await cc.setContractHash(contractHash);

  // console.log(`\n=====================================\n`);

  // /* Create master policy */
  // printHeader("Create Master Policy");

  // const createMasterPolicyDeploy = await cc.createMasterPolicy(
  //   "0",
  //   "master_policy",
  //   "0",
  //   20,
  //   "13000000000",
  //   KEYS.publicKey,
  //   [KEYS]
  // );

  // const createMasterPolicyDeployHash = await createMasterPolicyDeploy.send(
  //   NODE_URL!
  // );

  // console.log("...... Deploy hash: ", createMasterPolicyDeployHash);
  // console.log("...... Waiting for the deploy...");

  // await getDeploy(NODE_URL!, createMasterPolicyDeployHash);

  // console.log("Deploy Succedeed");

  // console.log("...... Querying Master Policy data...");

  // const [masterName, masterAssocCmp, masterReinsurencePerc] =
  //   await cc.queryMasterPolicy("0");

  // console.log(`.. name: ${masterName.value()}`);
  // console.log(`.. assocCmp: ${masterAssocCmp.value().toString()}`);
  // console.log(
  //   `.. reinsurence perc: ${masterReinsurencePerc.value().toString()}`
  // );

  // console.log(`\n=====================================\n`);

  // /* Create lap policy */
  // printHeader("Create Lap Policy");

  // const createLapPolicyDeploy = await cc.createLapPolicy(
  //   "0",
  //   "lap_policy",
  //   "0",
  //   "0",
  //   "data",
  //   "Uganda",
  //   "1000",
  //   "13000000000",
  //   KEYS.publicKey,
  //   [KEYS]
  // );

  // const createLapPolicyDeployHash = await createLapPolicyDeploy.send(NODE_URL!);

  // console.log("...... Deploy hash: ", createLapPolicyDeployHash);
  // console.log("...... Waiting for the deploy...");

  // await getDeploy(NODE_URL!, createLapPolicyDeployHash);

  // console.log("Deploy Succedeed");

  // console.log("...... Querying Lap Policy data...");

  // const [
  //   lapName,
  //   lapAssocMaster,
  //   lapLnpId,
  //   lapData,
  //   lapCountry,
  //   lapPremiumShare,
  // ] = await cc.queryLapPolicy("0");

  // console.log(`.. name: ${lapName.value()}`);
  // console.log(`.. assoc master: ${lapAssocMaster.value().toString()}`);
  // console.log(`.. lnp id: ${lapLnpId.value()}`);
  // console.log(`.. data: ${lapData.value()}`);
  // console.log(`.. country: ${lapCountry.value()}`);

  // console.log(`\n=====================================\n`);

  // /* Create Claim */
  // printHeader("Create Claim");

  // const createClaimDeploy = await cc.createClaim(
  //   "0",
  //   "0",
  //   "status",
  //   10000,
  //   "data",
  //   "adjusted_1000",
  //   "adjusted_notes",
  //   "3000000000",
  //   KEYS.publicKey,
  //   [KEYS]
  // );

  // const createClaimDeployHash = await createClaimDeploy.send(NODE_URL!);

  // console.log("...... Deploy hash: ", createClaimDeployHash);
  // console.log("...... Waiting for the deploy...");

  // await getDeploy(NODE_URL!, createClaimDeployHash);

  // console.log("Deploy Succedeed");

  // console.log("...... Querying Claim data...");

  // const [
  //   claimName,
  //   claimAssocLap,
  //   claimAmount,
  //   claimData,
  //   claimAdjustedAmount,
  //   claimAdjustedNotes,
  // ] = await cc.queryClaim("0");

  // console.log(`.. name: ${claimName.value()}`);
  // console.log(`.. assoc lap: ${claimAssocLap.value().toString()}`);
  // console.log(`.. amount: ${claimAmount.value()}`);
  // console.log(`.. data: ${claimData.value()}`);
  // console.log(`.. adjusted amount: ${claimAdjustedAmount.value()}`);
  // console.log(`.. adjusted notes: ${claimAdjustedNotes.value()}`);

  // console.log(`\n=====================================\n`);

  // /* Create Payment */
  // printHeader("Create Payment");

  // const createPaymentDeploy = await cc.createPaymentRecords(
  //   "0",
  //   "0",
  //   "payment_status",
  //   "10000",
  //   "payment_data",
  //   "13000000000",
  //   KEYS.publicKey,
  //   [KEYS]
  // );

  // const createPaymentDeployHash = await createPaymentDeploy.send(NODE_URL!);

  // console.log("...... Deploy hash: ", createPaymentDeployHash);
  // console.log("...... Waiting for the deploy...");

  // await getDeploy(NODE_URL!, createPaymentDeployHash);

  // console.log("Deploy Succedeed");

  // console.log("...... Querying Payment data...");

  // const [paymentName, paymentAssocClaim, paymentAmount, paymentData] =
  //   await cc.queryPaymentRecord("0");

  // console.log(`.. name: ${paymentName.value()}`);
  // console.log(`.. assoc lap: ${paymentAssocClaim.value().toString()}`);
  // console.log(`.. amount: ${paymentAmount.value()}`);
  // console.log(`.. data: ${paymentData.value()}`);

  // console.log(`\n=====================================\n`);

  // /* Create Premium */
  // printHeader("Create Premium Collection");

  // const createPremiumCollectionDeploy = await cc.createPremiumCollection(
  //   "0",
  //   "0",
  //   "premium_status",
  //   "10000",
  //   "premium_data",
  //   "13000000000",
  //   KEYS.publicKey,
  //   [KEYS]
  // );

  // const createPremiumCollectionDeployHash =
  //   await createPremiumCollectionDeploy.send(NODE_URL!);

  // console.log("...... Deploy hash: ", createPremiumCollectionDeployHash);
  // console.log("...... Waiting for the deploy...");

  // await getDeploy(NODE_URL!, createPremiumCollectionDeployHash);

  // console.log("Deploy Succedeed");

  // console.log("...... Querying Payment data...");

  // const [premiumName, premiumAssocLap, premiumAmount, premiumData] =
  //   await cc.queryPremiumCollection("0");

  // console.log(`.. name: ${premiumName.value()}`);
  // console.log(`.. assoc lap: ${premiumAssocLap.value().toString()}`);
  // console.log(`.. amount: ${premiumAmount.value()}`);
  // console.log(`.. data: ${premiumData.value()}`);

  // console.log(`\n=====================================\n`);

  // /* Create Premium */
  // printHeader("Update Claim");

  // const updateClaimDeploy = await cc.updateClaim(
  //   "0",
  //   "other_status",
  //   "adjusted_not1000",
  //   "notes_adjusted",
  //   "13000000000",
  //   KEYS.publicKey,
  //   [KEYS]
  // );

  // const updateClaimDeployHash = await updateClaimDeploy.send(NODE_URL!);

  // console.log("...... Deploy hash: ", updateClaimDeployHash);
  // console.log("...... Waiting for the deploy...");

  // await getDeploy(NODE_URL!, updateClaimDeployHash);

  // console.log("Deploy Succedeed");

  // console.log("...... Querying Payment data...");

  // const [
  //   updatedClaimName,
  //   updatedClaimAssocLap,
  //   updatedClaimAmount,
  //   updatedClaimData,
  //   updatedClaimAdjustedAmount,
  //   updatedClaimAdjustedNotes,
  // ] = await cc.queryClaim("0");

  // console.log(`.. updated name: ${updatedClaimName.value()}`);
  // console.log(
  //   `.. updated assoc lap: ${updatedClaimAssocLap.value().toString()}`
  // );
  // console.log(`.. updated amount: ${updatedClaimAmount.value()}`);
  // console.log(`.. updated data: ${updatedClaimData.value()}`);
  // console.log(
  //   `.. updated adjusted amount: ${updatedClaimAdjustedAmount.value()}`
  // );
  // console.log(
  //   `.. updated adjusted notes: ${updatedClaimAdjustedNotes.value()}`
  // );
};

run();

