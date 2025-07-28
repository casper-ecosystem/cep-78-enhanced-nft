import { expect, describe, it, beforeEach } from 'vitest';
import { RPC_URL, SSE_URL, CHAIN_NAME } from '../../config';
import {
  CEP78Client,
  MINTING_MODE,
  NFT_KIND,
  TransactionParams,
  TransferArgs,
  WHITELIST_MODE,
  EVENTS_MODE,
  HOLDER_MODE,
  IDENTIFIER_MODE,
  OWNERSHIP_MODE,
  NFT_METADATA_KIND,
  METADATA_MUTABILITY,
  OWNER_REVERSE_LOOKUP_MODE,
  BURN_MODE,
} from '../../src';
import { getAccountInfo, findKeyFromAccountNamedKeys } from '../utils';
import {
  install,
  owner,
  ali,
  mint,
  approve,
  generateTokenHash,
  bob,
  collectionConfig,
} from './helpers';

let client: CEP78Client;
const collectionName = `TEST_CEP78_E2E_${Math.floor(Math.random() * 1000000)}`;
const waitForTransactionProcessed = true;

describe('CEP78Client - E2E Usage', () => {
  beforeEach(async () => {
    client = new CEP78Client(RPC_URL, SSE_URL, CHAIN_NAME);
    await install(client, collectionName);
    const account = await getAccountInfo(RPC_URL, owner.publicKey),
      contractHash = findKeyFromAccountNamedKeys(
        account,
        `cep78_contract_hash_${collectionName}`
      );
    expect(contractHash).toBeDefined();
    client.setContractHash(contractHash);
  }, 180000);

  it('should mint tokens successfully', async () => {
    const initialBalance = (await client.balanceOf(owner.publicKey)) as string;

    const mintResult = await mint(client);

    expect(mintResult.transactionInfo.transactionHash).toBeDefined();
    expect(mintResult.executionResult?.errorMessage).toBeFalsy();

    const newBalance = (await client.balanceOf(owner.publicKey)) as string;
    expect(BigInt(newBalance)).toBe(BigInt(initialBalance) + BigInt(1));
  }, 180000);

  it('should transfer tokens successfully', async () => {
    const tokenHash = generateTokenHash();
    await mint(client, waitForTransactionProcessed, tokenHash);
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
  }, 180000);

  it('should approve and getApproved successfully', async () => {
    const tokenHash = generateTokenHash();
    await mint(client, waitForTransactionProcessed, tokenHash, ali);
    const approveResult = await approve(client, tokenHash);

    expect(approveResult.transactionInfo.transactionHash).toBeDefined();
    expect(approveResult.executionResult?.errorMessage).toBeFalsy();

    const approved = (await client.getApproved(tokenHash)) as string;
    expect(approved).toBe(bob.publicKey.accountHash().toPrefixedString());
  }, 180000);

  it('should burn tokens successfully', async () => {
    const tokenHash = generateTokenHash();
    await mint(client, waitForTransactionProcessed, tokenHash);

    const initialBalance = (await client.balanceOf(owner.publicKey)) as string;

    const burnResult = await client.burn({
      params: {
        sender: owner.publicKey,
        paymentAmount: String(5_000_000_000),
        signingKeys: [owner],
      },
      args: {
        tokenHash,
      },
      waitForTransactionProcessed: true,
    });

    expect(burnResult.transactionInfo.transactionHash).toBeDefined();
    expect(burnResult.executionResult?.errorMessage).toBeFalsy();

    const newBalance = (await client.balanceOf(owner.publicKey)) as string;
    expect(BigInt(newBalance)).toBe(BigInt(initialBalance) - BigInt(1));
  }, 180000);

  it('should register owner successfully', async () => {
    // This method is used to register an account as an NFT owner within the contract. Registering an owner may be a prerequisite for minting or receiving NFTs, depending on the contract's rules for OwnerReverseLookupMode (Complete/TransfersOnly).
    const registerResult = await client.register({
      params: {
        sender: bob.publicKey,
        paymentAmount: String(5_000_000_000),
        signingKeys: [bob],
      },
      args: {
        tokenOwner: bob.publicKey,
      },
      waitForTransactionProcessed: true,
    });

    expect(registerResult.transactionInfo.transactionHash).toBeDefined();
    expect(registerResult.executionResult?.errorMessage).toBeFalsy();
  }, 180000);

  it('should approve and then revoke an operator successfully', async () => {
    const tokenHash = generateTokenHash();
    await mint(client, waitForTransactionProcessed, tokenHash, ali);

    // Approve Bob as an operator for Ali's token
    const approveResult = await approve(client, tokenHash, ali, bob);

    expect(approveResult.transactionInfo.transactionHash).toBeDefined();
    expect(approveResult.executionResult?.errorMessage).toBeFalsy();

    // Verify that Bob is approved
    let approved = (await client.getApproved(tokenHash)) as string;
    expect(approved).toBe(bob.publicKey.accountHash().toPrefixedString());

    // Revoke approval
    const revokeResult = await client.revoke({
      params: {
        sender: ali.publicKey,
        paymentAmount: String(5_000_000_000),
        signingKeys: [ali],
      },
      args: {
        operator: bob.publicKey,
        tokenHash,
      },
      waitForTransactionProcessed: true,
    });

    expect(revokeResult.transactionInfo.transactionHash).toBeDefined();
    expect(revokeResult.executionResult?.errorMessage).toBeFalsy();

    // Confirm that Bob is no longer approved
    approved = (await client.getApproved(tokenHash)) as string;
    expect(approved).toBe('');
  }, 180000);

  it('should grant and revoke approval for all tokens successfully', async () => {
    const tokenHash = generateTokenHash();
    await mint(client, waitForTransactionProcessed, tokenHash, ali);

    // Grant Bob approval for all of Ali's NFTs
    const approveAllResult = await client.setApprovalForAll({
      params: {
        sender: ali.publicKey,
        paymentAmount: String(5_000_000_000),
        signingKeys: [ali],
      },
      args: {
        operator: bob.publicKey,
        approveAll: true,
      },
      waitForTransactionProcessed: true,
    });

    expect(approveAllResult.transactionInfo.transactionHash).toBeDefined();
    expect(approveAllResult.executionResult?.errorMessage).toBeFalsy();

    // Verify that Bob is an approved operator for Ali
    const isApproved = await client.isApprovedForAll({
      tokenOwner: ali.publicKey,
      operator: bob.publicKey,
    });
    expect(isApproved).toBe(true);

    // Revoke approval
    const revokeAllResult = await client.setApprovalForAll({
      params: {
        sender: ali.publicKey,
        paymentAmount: String(5_000_000_000),
        signingKeys: [ali],
      },
      args: {
        operator: bob.publicKey,
        approveAll: false,
      },
      waitForTransactionProcessed: true,
    });

    expect(revokeAllResult.transactionInfo.transactionHash).toBeDefined();
    expect(revokeAllResult.executionResult?.errorMessage).toBeFalsy();

    // Confirm Bob is no longer an approved operator
    const isApprovedAfterRevoke = await client.isApprovedForAll({
      tokenOwner: ali.publicKey,
      operator: bob.publicKey,
    });
    expect(isApprovedAfterRevoke).toBe(false);
  }, 180000);

  // Test is skipped as per https://github.com/casper-ecosystem/cep-78-enhanced-nft/issues/272
  it.skip('should update the token metadata successfully', async () => {
    const tokenId = '0';
    // Mint a new token first
    await mint(client, waitForTransactionProcessed, undefined, ali);

    // Define updated metadata
    const updatedMetadata = {
      ucid: tokenId,
      ipfs_cid: 'QmUpdatedIPFSHash987654321',
      color: 'Red',
    };

    // Update token metadata
    const updateMetadataResult = await client.setTokenMetadata({
      params: {
        sender: ali.publicKey,
        paymentAmount: String(5_000_000_000),
        signingKeys: [ali],
      },
      args: {
        tokenMetaData: updatedMetadata,
        tokenId,
      },
      waitForTransactionProcessed: true,
    });

    expect(updateMetadataResult.transactionInfo.transactionHash).toBeDefined();
    expect(updateMetadataResult.executionResult?.errorMessage).toBeFalsy();

    // Fetch and verify the updated metadata
    const fetchedMetadataRaw = await client.metadata(tokenId);
    const fetchedMetadata = JSON.parse(fetchedMetadataRaw);
    expect(fetchedMetadata).toEqual(updatedMetadata);
  }, 180000);

  it('should return the correct owner when querying directly via token hash', async () => {
    const tokenHash = generateTokenHash();
    await mint(client, waitForTransactionProcessed, tokenHash, ali);

    // Directly query the owner of the token using its hash
    const tokenOwner = await client.ownerOf(tokenHash);

    expect(tokenOwner).toBe(ali.publicKey.accountHash().toPrefixedString());
  }, 180000);

  it('should return the correct owner when querying and storing the result in an account named key', async () => {
    const tokenHash = generateTokenHash();
    await mint(client, waitForTransactionProcessed, tokenHash, bob);

    const keyName = 'stored_owner_of_token'; // Define a key name to store the result
    const params = {
      sender: ali.publicKey,
      paymentAmount: String(10_000_000_000), // 10 CSPR
      signingKeys: [ali],
    };

    // Store the owner of the token in the keyName via a session transaction
    const ownerOfArgs = {
      keyName,
      tokenHash: tokenHash, // Use the tokenHash as the identifier
    };

    await client.ownerOf({
      params,
      args: ownerOfArgs,
      waitForTransactionProcessed,
    });

    // Retrieve the account information for Ali to find the stored named key
    const aliAccountInfo = await getAccountInfo(RPC_URL, ali.publicKey);
    const storedOwnerOfValue = findKeyFromAccountNamedKeys(
      aliAccountInfo,
      keyName
    );

    expect(storedOwnerOfValue).toBeDefined(); // uref in acount context
  }, 180000);

  it('should return the correct balance when querying directly via token owner', async () => {
    const tokenHash = generateTokenHash();
    await mint(client, waitForTransactionProcessed, tokenHash, ali);

    // Directly query the balance of the token owner (Ali)
    const tokenOwnerBalance = await client.balanceOf(ali.publicKey);

    // Ensure the balance matches the expected value (in this case, Ali should have at least 1 token)
    expect(Number(tokenOwnerBalance)).toBeGreaterThan(0); // Assuming 1 token was minted for Ali
  }, 180000);

  it('should return the correct balance when querying and storing the result in an account named key', async () => {
    const tokenHash = generateTokenHash();
    await mint(client, waitForTransactionProcessed, tokenHash, bob);

    const keyName = 'stored_balance_of_token'; // Define a key name to store the result
    const params = {
      sender: ali.publicKey,
      paymentAmount: String(10_000_000_000), // 10 CSPR
      signingKeys: [ali],
    };

    // Store the balance of the token owner (Bob) in the keyName via a session transaction
    const balanceOfArgs = {
      keyName,
      tokenOwner: bob.publicKey,
    };

    await client.balanceOf({
      params,
      args: balanceOfArgs,
      waitForTransactionProcessed,
    });

    // Retrieve the account information for Ali to find the stored named key
    const aliAccountInfo = await getAccountInfo(RPC_URL, ali.publicKey);
    const storedBalanceOfValue = findKeyFromAccountNamedKeys(
      aliAccountInfo,
      keyName
    );

    expect(storedBalanceOfValue).toBeDefined(); // uref in account context
  }, 180000);

  it('should return false if the entity is not whitelisted in the ACL', async () => {
    const isWhitelisted = await client.isAclWhitelisted(bob.publicKey);
    expect(isWhitelisted).toBe(false);
  }, 180000);

  it('should return true if the entity is whitelisted in the ACL', async () => {
    const collectionName = `TEST_CEP78_E2E_${Math.floor(Math.random() * 1000000)}`;
    await install(client, collectionName, {
      mintingMode: MINTING_MODE.Acl,
      aclWhitelist: [owner.publicKey, ali.publicKey],
      whitelistMode: WHITELIST_MODE.Locked,
    });
    const account = await getAccountInfo(RPC_URL, owner.publicKey),
      contractHash = findKeyFromAccountNamedKeys(
        account,
        `cep78_contract_hash_${collectionName}`
      );
    expect(contractHash).toBeDefined();
    client.setContractHash(contractHash);

    let isWhitelisted = await client.isAclWhitelisted(ali.publicKey);

    // Ensure Ali is whitelisted
    expect(isWhitelisted).toBe(true);

    // Ensure Bob is not whitelisted
    isWhitelisted = await client.isAclWhitelisted(bob.publicKey);
    expect(isWhitelisted).toBe(false);
  }, 180000);

  it('should update contract variables correctly', async () => {
    const waitForTransactionProcessed = true;

    const params = {
      sender: owner.publicKey,
      paymentAmount: String(10_000_000_000), // 10 CSPR
      signingKeys: [owner],
    };

    const aclWhitelist = [owner.publicKey, bob.publicKey]; // List of whitelisted entities
    const allowMinting = true;
    const aclPackageMode = true;
    const packageOperatorMode = true;
    const operatorBurnMode = false;

    const setVariablesParams = {
      params,
      args: {
        allowMinting,
        aclWhitelist,
        aclPackageMode,
        packageOperatorMode,
        operatorBurnMode,
      },
      waitForTransactionProcessed,
    };

    const setVariablesResult = await client.setVariables(setVariablesParams);

    expect(setVariablesResult.transactionInfo.transactionHash).toBeDefined();
    expect(setVariablesResult.executionResult?.errorMessage).toBeFalsy();

    // Fetch updated contract variables directly after transaction
    const updatedAllowMinting = await client.allowMinting();
    const updatedAclPackageMode = await client.aclPackageMode();
    const updatedPackageOperatorMode = await client.packageOperatorMode();
    const updatedOperatorBurnMode = await client.operatorBurnMode();

    expect(updatedAllowMinting).toBe(allowMinting);
    expect(updatedAclPackageMode).toBe(aclPackageMode);
    expect(updatedPackageOperatorMode).toBe(packageOperatorMode);
    expect(updatedOperatorBurnMode).toBe(operatorBurnMode);

    // Check if bob is whitelisted
    const isWhitelisted = await client.isAclWhitelisted(bob.publicKey);
    expect(isWhitelisted).toBe(true);
  }, 180000);

  it('should return correct values for collection config', async () => {
    const collectionName = `TEST_CEP78_E2E_${Math.floor(Math.random() * 1000000)}`;
    await install(client, collectionName);
    const account = await getAccountInfo(RPC_URL, owner.publicKey),
      contractHash = findKeyFromAccountNamedKeys(
        account,
        `cep78_contract_hash_${collectionName}`
      );
    expect(contractHash).toBeDefined();
    client.setContractHash(contractHash);

    const [
      name,
      collectionSymbol,
      tokenTotalSupply,
      numOfMintedTokens,
      allowMinting,
      mintingMode,
      whitelistMode,
      reportingMode,
      burnMode,
      operatorBurnMode,
      holderMode,
      identifierMode,
      metadataMutability,
      nftKind,
      nftMetadataKind,
      ownershipMode,
      packageOperatorMode,
      aclPackageMode,
      jsonSchema,
      eventsMode,
    ] = await Promise.all([
      client.collectionName(),
      client.collectionSymbol(),
      client.tokenTotalSupply(),
      client.numOfMintedTokens(),
      client.allowMinting(),
      client.mintingMode(),
      client.whitelistMode(),
      client.reportingMode(),
      client.burnMode(),
      client.operatorBurnMode(),
      client.holderMode(),
      client.identifierMode(),
      client.metadataMutability(),
      client.nftKind(),
      client.nftMetadataKind(),
      client.ownershipMode(),
      client.packageOperatorMode(),
      client.aclPackageMode(),
      client.jsonSchema(),
      client.eventsMode(),
    ]);

    // Assertions for getters, matching the expected values in collectionConfig
    expect(name).toBe(collectionName);
    expect(collectionSymbol).toBe(collectionConfig.collectionSymbol);
    expect(tokenTotalSupply).toBe(collectionConfig.totalTokenSupply);
    expect(eventsMode).toBe(EVENTS_MODE[collectionConfig.eventsMode]);
    expect(holderMode).toBe(HOLDER_MODE[collectionConfig.holderMode]);
    expect(identifierMode).toBe(
      IDENTIFIER_MODE[collectionConfig.identifierMode]
    );
    expect(ownershipMode).toBe(OWNERSHIP_MODE[collectionConfig.ownershipMode]);
    expect(nftMetadataKind).toBe(
      NFT_METADATA_KIND[collectionConfig.nftMetadataKind]
    );
    expect(metadataMutability).toBe(
      METADATA_MUTABILITY[collectionConfig.metadataMutability]
    );
    expect(JSON.parse(jsonSchema)).toEqual(collectionConfig.jsonSchema);
    expect(numOfMintedTokens).toBeDefined();

    // default values
    expect(allowMinting).toBe(true);
    expect(mintingMode).toBe(MINTING_MODE[MINTING_MODE.Installer]);
    expect(whitelistMode).toBe(WHITELIST_MODE[WHITELIST_MODE.Unlocked]);
    expect(reportingMode).toBe(
      OWNER_REVERSE_LOOKUP_MODE[OWNER_REVERSE_LOOKUP_MODE.NoLookup]
    );
    expect(burnMode).toBe(BURN_MODE[BURN_MODE.Burnable]);
    expect(nftKind).toBe(NFT_KIND[NFT_KIND.Virtual]);
    expect(operatorBurnMode).toBe(false);
    expect(packageOperatorMode).toBe(false);
    expect(aclPackageMode).toBe(false);
  }, 180000);
});
