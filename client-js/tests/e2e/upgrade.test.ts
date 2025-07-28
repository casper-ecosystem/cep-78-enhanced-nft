import { expect, describe, it, beforeEach } from 'vitest';
import { owner, collectionConfig, install } from './helpers';
import { RPC_URL, SSE_URL, CHAIN_NAME, PRIVATE_KEY_FAUCET } from '../../config';
import {
  CEP78Client,
  EVENTS_MODE,
  TransactionParams,
  TransactionResult,
} from '../../src';
import wasm from '../../src/wasm/cep78';
import { getAccountInfo, findKeyFromAccountNamedKeys } from '../utils';

if (!PRIVATE_KEY_FAUCET) {
  throw new Error('FAUCET_SECRET_KEY environment variable is not set.');
}

let client: CEP78Client;
const collectionName = `TEST_CEP78_E2E_${Math.floor(Math.random() * 1000000)}`;

describe('CEP78Client - E2E Upgrade', () => {
  beforeEach(async () => {
    client = new CEP78Client(RPC_URL, SSE_URL, CHAIN_NAME);
    await install(client, collectionName);
  }, 180000);

  it('should upgrade the CEP78 contract and return valid transaction info', async () => {
    const params: TransactionParams = {
        wasm,
        sender: owner.publicKey,
        paymentAmount: collectionConfig.paymentAmount,
        signingKeys: [owner],
      },
      args = {
        collectionName,
        eventsMode: EVENTS_MODE.Native,
      },
      transactionResult: TransactionResult = await client.upgrade({
        params,
        args,
        waitForTransactionProcessed: false,
      });

    expect(
      transactionResult.transactionInfo.transactionHash.toHex()
    ).toBeTruthy();
  });

  it('should upgrade the CEP78 contract and return valid transaction result', async () => {
    const params: TransactionParams = {
        wasm,
        sender: owner.publicKey,
        paymentAmount: collectionConfig.paymentAmount,
        signingKeys: [owner],
      },
      args = {
        collectionName,
        eventsMode: EVENTS_MODE.Native,
      },
      transactionResult: TransactionResult = await client.upgrade({
        params,
        args,
        waitForTransactionProcessed: true,
      });

    expect(
      transactionResult.transactionInfo.transactionHash.toHex()
    ).toBeTruthy();
    expect(transactionResult.executionResult?.consumed).toBeTruthy();
    expect(transactionResult.executionResult?.errorMessage).toBeFalsy();

    const account = await getAccountInfo(RPC_URL, owner.publicKey);
    const contractHash = findKeyFromAccountNamedKeys(
      account,
      `cep78_contract_hash_${collectionName}`
    );
    expect(contractHash).toBeDefined();
    const contractPackageHash = findKeyFromAccountNamedKeys(
      account,
      `cep78_contract_package_${collectionName}`
    );
    expect(contractPackageHash).toBeDefined();
  }, 180000);
});
