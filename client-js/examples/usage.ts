import { sha256 } from '@noble/hashes/sha256';
import { bytesToHex } from '@noble/hashes/utils';
import {
  ExecutionResult,
  PublicKey,
  PutTransactionResult,
} from 'casper-js-sdk';
import { TextEncoder } from 'node:util';
import {
  PRIVATE_KEY_FAUCET,
  SSE_URL,
  PRIVATE_KEY_USER_1,
  CHAIN_NAME,
  RPC_URL,
} from '../config';
import {
  CEP78Client,
  MintArgs,
  OWNER_REVERSE_LOOKUP_MODE,
  TransactionParams,
  IDENTIFIER_MODE,
  RegisterArgs,
  TransferArgs,
  BurnArgs,
  TransactionResult,
  OwnerOfArgs,
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

const testCollectionName = 'TEST_CEP78',
  owner = getSigningKey(PRIVATE_KEY_FAUCET),
  ali = getSigningKey(PRIVATE_KEY_USER_1),
  waitForTransactionProcessed = true;

const usage = async () => {
  const account = await getAccountInfo(RPC_URL, owner.publicKey),
    contractHash = findKeyFromAccountNamedKeys(
      account,
      `cep78_contract_hash_${testCollectionName}`
    );

  const cep78 = new CEP78Client(RPC_URL, SSE_URL, CHAIN_NAME).setContractHash(
    contractHash
  );
  console.info(`Contract Hash: ${cep78.contractHash.toPrefixedString()}`);

  // Fetch some token info
  const collectionName = await cep78.collectionName(),
    symbol = await cep78.collectionSymbol(),
    tokenTotalSupply = await cep78.tokenTotalSupply(),
    allowMinting = await cep78.allowMinting(),
    mintingMode = await cep78.mintingMode(),
    burnMode = await cep78.burnMode(),
    holderMode = await cep78.holderMode(),
    identifierMode = await cep78.identifierMode(),
    whitelistMode = await cep78.whitelistMode(),
    ownerReverseLookupMode = await cep78.reportingMode(),
    metadataMutability = await cep78.metadataMutability();

  console.info('Collection info:', {
    collectionName,
    symbol,
    tokenTotalSupply: tokenTotalSupply.toString(),
    holderMode,
    allowMinting,
    mintingMode,
    whitelistMode,
    burnMode,
    identifierMode,
    metadataMutability,
    ownerReverseLookupMode,
  });

  const callSessionWasm =
    ownerReverseLookupMode ===
    OWNER_REVERSE_LOOKUP_MODE[OWNER_REVERSE_LOOKUP_MODE.Complete];

  const mintArgs: MintArgs = {
    tokenOwner: owner.publicKey,
    tokenMetaData: {
      ipfs_cid: 'QmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR',
      color: 'Blue',
    },
  };

  let tokenIdentifier: string;
  if (identifierMode === IDENTIFIER_MODE[IDENTIFIER_MODE.Hash]) {
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
    tokenIdentifier = `${+(await cep78.numOfMintedTokens())}`;
  }

  mintArgs.tokenMetaData['ucid'] = tokenIdentifier;

  const ownerIsWhiteListed = await cep78.isAclWhitelisted(owner.publicKey);

  if (!ownerIsWhiteListed) {
    throw new Error('Owner is not whitelisted');
  }

  console.info(`Mint token ${tokenIdentifier}`);

  let params = {
    sender: owner.publicKey,
    paymentAmount: String(5_000_000_000), // 5 CSPR
    signingKeys: [owner],
  };

  await executeTransaction(
    'mint',
    cep78,
    params,
    mintArgs,
    waitForTransactionProcessed,
    callSessionWasm
  );
  await printTokenDetails(cep78, owner.publicKey, tokenIdentifier);

  console.info('Register');
  params = {
    sender: ali.publicKey,
    paymentAmount: String(2_500_000_000), // 2.5 CSPR
    signingKeys: [ali],
  };

  await executeTransaction(
    'register',
    cep78,
    params,
    { tokenOwner: ali.publicKey },
    waitForTransactionProcessed
  );

  console.info('Transfer');
  params = {
    sender: owner.publicKey,
    paymentAmount: String(5_000_000_000), // 5 CSPR
    signingKeys: [owner],
  };

  const transferArgs = {
    source: owner.publicKey,
    target: ali.publicKey,
    ...(identifierMode === IDENTIFIER_MODE[IDENTIFIER_MODE.Hash]
      ? { tokenHash: tokenIdentifier }
      : { tokenId: tokenIdentifier }),
  };

  await executeTransaction(
    'transfer',
    cep78,
    params,
    transferArgs,
    waitForTransactionProcessed,
    callSessionWasm
  );
  await printTokenDetails(cep78, ali.publicKey, tokenIdentifier);

  // Store owner of at account named key
  console.info(`Store owner of token ${tokenIdentifier}`);

  params = {
    sender: ali.publicKey,
    paymentAmount: String(2_500_000_000), // 2.5 CSPR
    signingKeys: [ali],
  };

  // Store ownerOfArgs, call session client contract and store to keyName
  const keyName = 'stored_owner_of_token';

  const ownerOfArgs: OwnerOfArgs = {
    keyName,
    ...(identifierMode === IDENTIFIER_MODE[IDENTIFIER_MODE.Hash]
      ? { tokenHash: tokenIdentifier }
      : { tokenId: tokenIdentifier }),
  };

  await executeTransaction(
    'ownerOf',
    cep78,
    params,
    ownerOfArgs,
    waitForTransactionProcessed
  );

  // Getting ali's account namedKeys, value was stored as temp data and may not reflect actual global state,
  // specially after next burn action
  const aliAccountInfo = await getAccountInfo(RPC_URL, ali.publicKey);
  const storedOwnerOfValue = findKeyFromAccountNamedKeys(
    aliAccountInfo,
    keyName
  );

  console.info(`Stored '${keyName}' value at URef: ${storedOwnerOfValue}`);

  console.info('Burn');
  params = {
    sender: ali.publicKey,
    paymentAmount: String(2_500_000_000), // 2.5 CSPR
    signingKeys: [ali],
  };

  const burnArgs: { tokenHash?: string; tokenId?: string } =
    identifierMode === IDENTIFIER_MODE[IDENTIFIER_MODE.Hash]
      ? { tokenHash: tokenIdentifier }
      : { tokenId: tokenIdentifier };

  await executeTransaction(
    'burn',
    cep78,
    params,
    burnArgs,
    waitForTransactionProcessed
  );
};

async function executeTransaction(
  action: 'mint' | 'register' | 'transfer' | 'burn' | 'ownerOf',
  cep78: CEP78Client,
  params: TransactionParams,
  args: MintArgs | RegisterArgs | TransferArgs | BurnArgs | OwnerOfArgs,
  waitForTransactionProcessed?: boolean,
  callSessionWasm?: boolean
): Promise<void> {
  let transactionInfo: PutTransactionResult,
    executionResult: ExecutionResult | undefined;

  switch (action) {
    case 'mint':
      ({ transactionInfo, executionResult } = await cep78.mint(
        {
          params,
          args: args as MintArgs,
          waitForTransactionProcessed,
        },
        callSessionWasm
      ));
      break;
    case 'register':
      ({ transactionInfo, executionResult } = await cep78.register({
        params,
        args: args as RegisterArgs,
        waitForTransactionProcessed,
      }));
      break;
    case 'transfer':
      ({ transactionInfo, executionResult } = await cep78.transfer(
        {
          params,
          args: args as TransferArgs,
          waitForTransactionProcessed,
        },
        callSessionWasm
      ));
      break;
    case 'burn':
      ({ transactionInfo, executionResult } = await cep78.burn({
        params,
        args: args as BurnArgs,
        waitForTransactionProcessed,
      }));
      break;
    case 'ownerOf':
      ({ transactionInfo, executionResult } = (await cep78.ownerOf({
        params,
        args: args as OwnerOfArgs,
        waitForTransactionProcessed,
      })) as TransactionResult);
      break;
    default:
      throw new Error(`Unknown action: ${action}`);
  }

  if (executionResult?.errorMessage) {
    throw new Error(
      `Error during ${action}.\n${executionResult?.errorMessage.toString()}`
    );
  } else {
    console.info(
      `${action.charAt(0).toUpperCase() + action.slice(1)} transaction hash: ${transactionInfo.transactionHash.toHex()}`
    );
    console.info(
      `${action.charAt(0).toUpperCase() + action.slice(1)} cost consumed: ${executionResult?.consumed}`
    );
  }
}

const printTokenDetails = async (
  cep78: CEP78Client,
  account: PublicKey,
  tokenIdentifier: string
) => {
  const ownerBalance = (await cep78.balanceOf(account)) as string;
  console.info(`Account ${account} balance ${ownerBalance}`);

  const tokenOwner = (await cep78.ownerOf(tokenIdentifier)) as string;
  console.info(`Owner of token ${tokenIdentifier} is ${tokenOwner}`);

  const metadata = (await cep78.metadata(tokenIdentifier)) as unknown;
  console.info(`Metadata:`, metadata);
};

usage()
  .then(() => {
    console.info('Usage completed successfully.');
  })
  .catch((error) => {
    console.error('Usage failed:', error);
  });
