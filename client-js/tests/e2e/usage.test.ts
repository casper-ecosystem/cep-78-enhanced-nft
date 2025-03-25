import { expect, describe, it, beforeEach } from 'vitest';
import { RPC_URL, SSE_URL, CHAIN_NAME } from '../../config';
import { CEP78Client, TransactionParams, TransferArgs } from '../../src';
import { getAccountInfo, findKeyFromAccountNamedKeys } from '../utils';
import { install, owner, ali, mint, approve } from './helpers';

let client: CEP78Client;
const name = `TEST_CEP78_E2E_${Math.floor(Math.random() * 1000000)}`,
  tokenHash = 'tokenHash';

describe('CEP78Client - E2E Usage', () => {
  beforeEach(async () => {
    client = new CEP78Client(RPC_URL, SSE_URL, CHAIN_NAME);
    await install(client, name);
    const account = await getAccountInfo(RPC_URL, owner.publicKey),
      contractHash = findKeyFromAccountNamedKeys(
        account,
        `cep78_contract_hash_${name}`
      );
    expect(contractHash).toBeDefined();
    client.setContractHash(contractHash);
    await mint(client);
  }, 60000);

  it('should transfer tokens successfully', async () => {
    const initialBalance = (await client.balanceOf(owner.publicKey)) as string,
      initialBalanceAli = (await client.balanceOf(ali.publicKey)) as string,
      params: TransactionParams = {
        sender: owner.publicKey,
        paymentAmount: String(5_000_000_000),
        signingKeys: [owner],
      },
      transferArgs: TransferArgs = {
        target: ali.publicKey,
        source: owner.publicKey,
        tokenHash,
      },
      transferResult = await client.transfer({
        params,
        args: transferArgs,
        waitForTransactionProcessed: true,
      });

    expect(transferResult.transactionInfo.transactionHash).toBeDefined();
    expect(transferResult.executionResult?.errorMessage).toBeFalsy();

    const newBalance = (await client.balanceOf(owner.publicKey)) as string;
    expect(BigInt(newBalance)).toBe(BigInt(initialBalance) - BigInt(1));

    const newBalanceAli = (await client.balanceOf(ali.publicKey)) as string;
    expect(BigInt(newBalanceAli)).toBe(BigInt(initialBalanceAli) + BigInt(1));
  }, 60000);

  it('should getApproved successfully', async () => {
    const approveResult = await approve(client);

    expect(approveResult.transactionInfo.transactionHash).toBeDefined();
    expect(approveResult.executionResult?.errorMessage).toBeFalsy();

    const approved = await client.getApproved(tokenHash);
    expect(approved).toBeDefined();
  }, 60000);

  it('should mint tokens successfully', async () => {
    const initialBalance = (await client.balanceOf(owner.publicKey)) as string;

    const mintResult = await mint(client, 'newTokenHash');

    expect(mintResult.transactionInfo.transactionHash).toBeDefined();
    expect(mintResult.executionResult?.errorMessage).toBeFalsy();

    const newBalance = (await client.balanceOf(owner.publicKey)) as string;
    expect(BigInt(newBalance)).toBe(BigInt(initialBalance) + BigInt(1));
  }, 60000);

  it('should burn tokens successfully', async () => {
    await mint(client, 'newTokenHashToBurn');

    const initialBalance = (await client.balanceOf(owner.publicKey)) as string;

    const burnResult = await client.burn({
      params: {
        sender: owner.publicKey,
        paymentAmount: String(5_000_000_000),
        signingKeys: [owner],
      },
      args: {
        tokenHash: 'newTokenHashToBurn',
      },
      waitForTransactionProcessed: true,
    });

    expect(burnResult.transactionInfo.transactionHash).toBeDefined();
    expect(burnResult.executionResult?.errorMessage).toBeFalsy();

    const newBalance = (await client.balanceOf(ali.publicKey)) as string;
    expect(BigInt(newBalance)).toBe(BigInt(initialBalance) - BigInt(1));
  }, 60000);
});
