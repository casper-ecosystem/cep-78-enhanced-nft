import { expect, describe, it, beforeEach } from 'vitest';
import { RPC_URL, SSE_URL, CHAIN_NAME } from '../../config';
import { CEP78Client, TransactionParams, TransactionResult } from '../../src';
import wasm from '../../src/wasm/cep78';
import { getAccountInfo, findKeyFromAccountNamedKeys } from '../utils';
import { owner, install, collectionConfig } from './helpers';

let client: CEP78Client;

describe('CEP78Client - E2E Install', () => {
  beforeEach(() => {
    client = new CEP78Client(RPC_URL, SSE_URL, CHAIN_NAME);
  });

  it('should install the CEP78 contract and return valid transaction info', async () => {
    const collectionName = `TEST_CEP78_E2E_${Math.floor(Math.random() * 1000000)}`,
      params: TransactionParams = {
        wasm,
        sender: owner.publicKey,
        paymentAmount: collectionConfig.paymentAmount,
        signingKeys: [owner],
      },
      args = {
        collectionName,
        ...collectionConfig,
      },
      transactionResult: TransactionResult = await client.install({
        params,
        args,
        waitForTransactionProcessed: false,
      });

    expect(
      transactionResult.transactionInfo.transactionHash.toHex()
    ).toBeTruthy();
  });

  it('should install the CEP78 contract and return valid transaction result', async () => {
    const collectionName = `TEST_CEP78_E2E_${Math.floor(Math.random() * 1000000)}`,
      transactionResult: TransactionResult = await install(
        client,
        collectionName
      );

    expect(
      transactionResult.transactionInfo.transactionHash.toHex()
    ).toBeTruthy();
    expect(transactionResult.executionResult?.consumed).toBeTruthy();
    expect(transactionResult.executionResult?.errorMessage).toBeFalsy();

    // Check for the contract hash after installation
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
