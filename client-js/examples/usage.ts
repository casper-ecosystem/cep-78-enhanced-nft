import { sha256 } from '@noble/hashes/sha256';
import { bytesToHex } from '@noble/hashes/utils';
import { PublicKey } from 'casper-js-sdk';
import { TextEncoder } from 'node:util';
import {
  PRIVATE_KEY_FAUCET,
  SSE_URL,
  PRIVATE_KEY_USER_1,
  PRIVATE_KEY_USER_2,
  CHAIN_NAME,
  RPC_URL,
} from '../config';
import {
  CEP78Client,
  MintArgs,
  OWNER_REVERSE_LOOKUP_MODE,
  TransactionParams,
  NFT_IDENTIFIER_MODE,
  RegisterArgs,
  TransferArgs,
  BurnArgs,
} from '../dist';
import {
  findKeyFromAccountNamedKeys,
  getAccountInfo,
  getSigningKey,
} from '../tests/utils';

// Here you can check examples how to check balance, approve tokens, transfer tokens, and transfer tokens by allowance

if (!PRIVATE_KEY_FAUCET) {
  throw new Error('FAUCET_SECRET_KEY environment variable is not set.');
}
if (!PRIVATE_KEY_USER_1) {
  throw new Error('PRIVATE_KEY_USER_1 environment variable is not set.');
}
if (!PRIVATE_KEY_USER_2) {
  throw new Error('PRIVATE_KEY_USER_2 environment variable is not set.');
}

const testCollectionName = 'TEST_CEP78',
  owner = getSigningKey(PRIVATE_KEY_FAUCET),
  ali = getSigningKey(PRIVATE_KEY_USER_1),
  waitForTransactionProcessed = true;

const usage = async () => {
  const accountInfo = await getAccountInfo(RPC_URL, owner.publicKey),
    contractHash = findKeyFromAccountNamedKeys(
      accountInfo,
      `cep78_contract_hash_${testCollectionName}`
    );

  const cep78 = new CEP78Client(RPC_URL, SSE_URL, CHAIN_NAME).setContractHash(
    contractHash
  );
  console.info(`Contract Hash: ${cep78.contractHash.toPrefixedString()}`);

  // Fetch token info
  const collectionName = await cep78.collectionName(),
    symbol = await cep78.collectionSymbol(),
    tokenTotalSupply = await cep78.tokenTotalSupply(),
    allowMinting = await cep78.allowMinting(),
    burnMode = await cep78.burnMode(),
    holderMode = await cep78.holderMode(),
    identifierMode = await cep78.identifierMode(),
    whitelistMode = await cep78.whitelistMode(),
    ownerReverseLookupMode = await cep78.reportingMode();

  console.info('Collection info:', {
    collectionName,
    symbol,
    tokenTotalSupply: tokenTotalSupply.toString(),
    allowMinting,
    burnMode,
    holderMode,
    identifierMode,
    whitelistMode,
    ownerReverseLookupMode,
  });

  const callSessionWasm =
    ownerReverseLookupMode ===
    OWNER_REVERSE_LOOKUP_MODE[OWNER_REVERSE_LOOKUP_MODE.Complete];

  const mintArgs: MintArgs = {
    collectionName: 'my-collection',
    tokenOwner: owner.publicKey,
    tokenMetaData: {
      ipfs_cid: 'QmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR',
      color: 'Blue',
    },
  };

  let tokenIdentifier: string;
  if (identifierMode === NFT_IDENTIFIER_MODE[NFT_IDENTIFIER_MODE.Hash]) {
    tokenIdentifier = bytesToHex(
      sha256(
        new TextEncoder().encode(
          `my_custom_token_hash_${Math.floor(Math.random() * 1000000)}`
        )
      )
    );
    mintArgs.tokenHash = tokenIdentifier;
  } else {
    // If token identifier is not a custom hash or given token id, assume token id is owner current balance
    tokenIdentifier = `${+(await cep78.balanceOf(owner.publicKey))}`;
  }

  mintArgs.tokenMetaData['ucid'] = tokenIdentifier;

  console.info(`Mint token ${tokenIdentifier}`);

  let params: TransactionParams = {
    sender: owner.publicKey,
    paymentAmount: String(5_000_000_000),
    signingKeys: [owner],
  };

  let { transactionInfo, executionResult } = await cep78.mint(
    {
      params,
      args: mintArgs,
      waitForTransactionProcessed,
    },
    callSessionWasm
  );

  if (executionResult?.errorMessage) {
    throw new Error(
      `Error during mint.\n${executionResult?.errorMessage.toString()}`
    );
  } else {
    console.info(
      `Token mint transaction hash: ${transactionInfo.transactionHash}`
    );
    console.info(`Mint cost consumed: ${executionResult?.consumed}`);
  }

  await printTokenDetails(cep78, owner.publicKey, tokenIdentifier);

  console.info('Register');

  params = {
    sender: ali.publicKey,
    paymentAmount: String(310_000_000),
    signingKeys: [ali],
  };

  const registerArgs: RegisterArgs = {
    tokenOwner: ali.publicKey,
  };

  ({ transactionInfo, executionResult } = await cep78.register({
    params,
    args: registerArgs,
    waitForTransactionProcessed,
  }));

  if (executionResult?.errorMessage) {
    throw new Error(
      `Error during register.\n${executionResult?.errorMessage.toString()}`
    );
  } else {
    console.info(
      `Owner register transaction hash: ${transactionInfo.transactionHash}`
    );
    console.info(`Register cost consumed: ${executionResult?.consumed}`);
  }

  console.info('Transfer');

  params = {
    sender: owner.publicKey,
    paymentAmount: String(4_500_000_000),
    signingKeys: [owner],
  };

  const transferArgs: TransferArgs = {
    source: owner.publicKey,
    target: ali.publicKey,
  };

  if (identifierMode === NFT_IDENTIFIER_MODE[NFT_IDENTIFIER_MODE.Hash]) {
    transferArgs.tokenHash = tokenIdentifier;
  } else {
    transferArgs.tokenId = tokenIdentifier;
  }

  ({ transactionInfo, executionResult } = await cep78.transfer({
    params,
    args: transferArgs,
    waitForTransactionProcessed,
  }));

  if (executionResult?.errorMessage) {
    throw new Error(
      `Error during transfer.\n${executionResult?.errorMessage.toString()}`
    );
  } else {
    console.info(
      `Transfer transaction hash: ${transactionInfo.transactionHash}`
    );
    console.info(`Transfer cost consumed: ${executionResult?.consumed}`);
  }

  await printTokenDetails(cep78, ali.publicKey, tokenIdentifier);

  /* Burn */
  console.info('Burn');

  params = {
    sender: ali.publicKey,
    paymentAmount: String(1_000_000_000),
    signingKeys: [ali],
  };

  const burnArgs: BurnArgs = {};
  if (identifierMode === NFT_IDENTIFIER_MODE[NFT_IDENTIFIER_MODE.Hash]) {
    burnArgs.tokenHash = tokenIdentifier;
  } else {
    burnArgs.tokenId = tokenIdentifier;
  }

  ({ transactionInfo, executionResult } = await cep78.burn({
    params,
    args: burnArgs,
    waitForTransactionProcessed,
  }));

  if (executionResult?.errorMessage) {
    throw new Error(
      `Error during burn.\n${executionResult?.errorMessage.toString()}`
    );
  } else {
    console.info(`Burn transaction hash: ${transactionInfo.transactionHash}`);
    console.info(`Burn cost consumed: ${executionResult?.consumed}`);
  }
};

const printTokenDetails = async (
  cep78: CEP78Client,
  account: PublicKey,
  tokenIdentifier: string
) => {
  const ownerBalance = await cep78.balanceOf(account);
  console.info(`Account ${account} balance ${ownerBalance}`);

  const tokenOwner = await cep78.ownerOf(tokenIdentifier);
  console.info(`Owner of token ${tokenIdentifier} is ${tokenOwner}`);

  const metadata = await cep78.metadata(tokenIdentifier);
  console.info(`Metadata:`, metadata);
};

usage()
  .then(() => {
    console.info('Usage completed successfully.');
  })
  .catch((error) => {
    console.error('Usage failed:', error);
  });
