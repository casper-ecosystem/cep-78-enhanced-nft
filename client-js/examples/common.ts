// eslint-disable-next-line eslint-comments/disable-enable-pair
/* eslint-disable no-console */
import { config } from "dotenv";
import {
  Keys,
  CLPublicKey,
  CasperClient,
  CasperServiceByJsonRPC,
} from "casper-js-sdk";

import * as fs from "fs";

config();

const { MASTER_KEY_PAIR_PATH, USER1_KEY_PAIR_PATH, USER2_KEY_PAIR_PATH } =
  process.env;

export const FAUCET_KEYS = Keys.Ed25519.parseKeyFiles(
  `${MASTER_KEY_PAIR_PATH}/public_key.pem`,
  `${MASTER_KEY_PAIR_PATH}/secret_key.pem`
);

export const USER1_KEYS = Keys.Ed25519.parseKeyFiles(
  `${USER1_KEY_PAIR_PATH}/public_key.pem`,
  `${USER1_KEY_PAIR_PATH}/secret_key.pem`
);

export const USER2_KEYS = Keys.Ed25519.parseKeyFiles(
  `${USER2_KEY_PAIR_PATH}/public_key.pem`,
  `${USER2_KEY_PAIR_PATH}/secret_key.pem`
);

export const getBinary = (pathToBinary: string) => new Uint8Array(fs.readFileSync(pathToBinary, null).buffer);

export const sleep = (ms: number) => new Promise<void>((resolve) => {
  setTimeout(() => resolve(), ms);
});

//! TODO refacto to listen events
export const getDeploy = async (nodeURL: string, deployHash: string, retries = 300): Promise<unknown> => {
  const client = new CasperClient(nodeURL);
  const [deploy, raw] = await client.getDeploy(deployHash);

  if (raw.execution_results.length !== 0) {
    if (raw.execution_results[0].result.Success) {
      return deploy;
    }
    throw new Error(
      `Contract execution: ${raw.execution_results[0].result.Failure.error_message}`
    );
  }

  if (retries <= 0) {
    throw new Error(`Timeout after 300s. Something's wrong`);
  }

  await sleep(1000);
  return getDeploy(nodeURL, deployHash, retries - 1);
};

export const getAccountInfo = async (
  nodeAddress: string,
  publicKey: CLPublicKey
) => {
  const client = new CasperServiceByJsonRPC(nodeAddress);
  const stateRootHash = await client.getStateRootHash();
  const accountHash = publicKey.toAccountHashStr();
  const blockState = await client.getBlockState(stateRootHash, accountHash, []);
  return blockState.Account as AccountInfo;
};

/**
 * Returns a value under an on-chain account's storage.
 * @param accountInfo - On-chain account's info.
 * @param namedKey - A named key associated with an on-chain account.
 */
export const getAccountNamedKeyValue = (accountInfo: AccountInfo, namedKey: string): string | undefined => {
  const found = accountInfo.namedKeys.find((i) => i.name === namedKey);
  return found?.key;
};

export const printHeader = (text: string) => {
  console.log(`******************************************`);
  console.log(`* ${text} *`);
  console.log(`******************************************`);
};


interface NamedKey {
  name: string;
  key: string;
}

export type AccountInfo = {
  namedKeys: NamedKey[];
};