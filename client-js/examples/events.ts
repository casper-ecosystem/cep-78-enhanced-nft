import { sha256 } from '@noble/hashes/sha256';
import { bytesToHex } from '@noble/hashes/utils';
import {
  type MintArgs,
  CEP78Client,
  CEP78_EVENTS,
  CEP78EventResult,
  InfoGetTransactionResult,
  OWNER_REVERSE_LOOKUP_MODE,
  IDENTIFIER_MODE,
  ApproveArgs,
  SetApprovallForAllArgs,
  IsApprovedForAllArgs,
  IsApprovedForAlldParams,
} from 'dist';
import { TextEncoder } from 'node:util';
import {
  PRIVATE_KEY_FAUCET,
  SSE_URL,
  PRIVATE_KEY_USER_1,
  CHAIN_NAME,
  RPC_URL,
  PRIVATE_KEY_USER_2,
} from '../config';
import {
  findKeyFromAccountNamedKeys,
  getAccountInfo,
  getSigningKey,
} from '../tests/utils';
// Here you can check examples how to mint and burn tokens and listen to event stream

if (!PRIVATE_KEY_FAUCET) {
  throw new Error('FAUCET_SECRET_KEY environment variable is not set.');
}
if (!PRIVATE_KEY_USER_1) {
  throw new Error('PRIVATE_KEY_USER_1 environment variable is not set.');
}
if (!PRIVATE_KEY_USER_2) {
  throw new Error('PRIVATE_KEY_USER_2 environment variable is not set.');
}

const collectionName = 'TEST_CEP78',
  owner = getSigningKey(PRIVATE_KEY_FAUCET),
  ali = getSigningKey(PRIVATE_KEY_USER_1),
  bob = getSigningKey(PRIVATE_KEY_USER_2);

const usage = async () => {
  const account = await getAccountInfo(RPC_URL, owner.publicKey),
    contractHash = findKeyFromAccountNamedKeys(
      account,
      `cep78_contract_hash_${collectionName}`
    );

  const cep78 = new CEP78Client(RPC_URL, SSE_URL, CHAIN_NAME)
    .setContractHash(contractHash)
    .startEventStream();

  console.info(`Contract Hash: ${cep78.contractHash.toPrefixedString()}`);

  if (!(await cep78.allowMinting())) {
    console.warn(`Mint is disabled.`);
    return;
  }

  const callSessionWasm =
    (await cep78.reportingMode()) ===
    OWNER_REVERSE_LOOKUP_MODE[OWNER_REVERSE_LOOKUP_MODE.Complete];

  let params = {
    sender: owner.publicKey,
    paymentAmount: String(5_000_000_000), // 5 CSPR
    signingKeys: [owner],
  };

  const mintArgs: MintArgs = {
    tokenOwner: ali.publicKey,
    tokenMetaData: {
      ipfs_cid: 'QmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR',
      color: 'Blue',
    },
  };

  const identifierMode = await cep78.identifierMode();
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

  // Mint
  console.info(`Mint token ${tokenIdentifier}`);

  await cep78.mint(
    {
      params,
      args: mintArgs,
    },
    callSessionWasm
  );

  const mintEvent = CEP78_EVENTS.Mint;
  await new Promise<void>((resolve, reject) => {
    cep78.on(mintEvent, async (eventResult) => {
      try {
        await eventListener(cep78, mintEvent, eventResult);
        resolve();
      } catch (error) {
        reject(error);
      }
    });
  });

  let aliBalance = await cep78.balanceOf(ali.publicKey);
  console.info(
    `Token minted successfully, Ali's balance: ${aliBalance.toString()}`
  );

  // Approve
  console.info(`Approval for Bob ${bob.publicKey}`);

  params = {
    sender: ali.publicKey,
    paymentAmount: String(2_500_000_000), // 2.5 CSPR
    signingKeys: [ali],
  };

  const approveArgs: ApproveArgs = {
    operator: bob.publicKey,
    ...(identifierMode === IDENTIFIER_MODE[IDENTIFIER_MODE.Hash]
      ? { tokenHash: tokenIdentifier }
      : { tokenId: tokenIdentifier }),
  };

  await cep78.approve({
    params,
    args: approveArgs,
  });

  const approveEvent = CEP78_EVENTS.Approval;
  await new Promise<void>((resolve, reject) => {
    cep78.on(approveEvent, async (eventResult) => {
      try {
        await eventListener(cep78, approveEvent, eventResult);
        resolve();
      } catch (error) {
        reject(error);
      }
    });
  });

  const approved = (await cep78.ownerOf(tokenIdentifier)) as string;

  console.info(`Approved account for token ${tokenIdentifier}: ${approved}`);

  // Clean subscriptions, either the stream is running or you're waiting for a transaction
  cep78.stopEventStream();

  // Transfer
  console.info(`Transfer token ${tokenIdentifier}`);
  params = {
    sender: bob.publicKey,
    paymentAmount: String(5_000_000_000), // 5 CSPR
    signingKeys: [bob],
  };

  const transferArgs = {
    source: ali.publicKey,
    target: bob.publicKey,
    ...(identifierMode === IDENTIFIER_MODE[IDENTIFIER_MODE.Hash]
      ? { tokenHash: tokenIdentifier }
      : { tokenId: tokenIdentifier }),
  };

  let { transactionInfo } = await cep78.transfer({
    params,
    args: transferArgs,
    waitForTransactionProcessed: true,
  });

  console.info(
    `Contract  Transfer transaction hash: ${transactionInfo.transactionHash.toHex()}`
  );

  const bobBalance = await cep78.balanceOf(bob.publicKey);
  aliBalance = await cep78.balanceOf(ali.publicKey);
  console.info(
    `Token transfer successfully, Bob's balance: ${bobBalance.toString()}, Ali's balance: ${aliBalance.toString()}`
  );

  // SetApprovallForAll
  params = {
    sender: bob.publicKey,
    paymentAmount: String(2_500_000_000), // 2.5 CSPR
    signingKeys: [bob],
  };

  const setApprovallForAllArgs: SetApprovallForAllArgs = {
    operator: owner.publicKey,
    approveAll: true,
  };

  ({ transactionInfo } = await cep78.setApprovalForAll({
    params,
    args: setApprovallForAllArgs,
    waitForTransactionProcessed: true,
  }));

  console.info(
    `Contract SetApprovalForAll transaction hash: ${transactionInfo.transactionHash.toHex()}`
  );

  // IsApprovedForAlldParams, query global state dictionary
  const isApprovedForAlldParams: IsApprovedForAlldParams = {
    tokenOwner: bob.publicKey,
    operator: owner.publicKey,
  };

  const isApprovedForAll = await cep78.isApprovedForAll(
    isApprovedForAlldParams
  );

  console.info(
    `Owner is approved for all token from Bob ${bob.publicKey}: ${isApprovedForAll}`
  );

  // Store IsApprovedForAlldParams, call session client contract and store to keyName
  const keyName = 'test_is_approved_for_all';

  const isApprovedForAllArgs: IsApprovedForAllArgs = {
    ...isApprovedForAlldParams,
    keyName,
  };

  params = {
    sender: owner.publicKey,
    paymentAmount: String(2_500_000_000), // 2.5 CSPR
    signingKeys: [owner],
  };

  await cep78.isApprovedForAll({
    params,
    args: isApprovedForAllArgs,
    waitForTransactionProcessed: true,
  });

  const owner_account = await getAccountInfo(RPC_URL, owner.publicKey);
  const storedValue = findKeyFromAccountNamedKeys(owner_account, keyName);

  console.info(`Stored '${keyName}' value at URef: ${storedValue}`);
};

const eventListener = async (
  cep78: CEP78Client,
  eventType: keyof typeof CEP78_EVENTS,
  eventResult: CEP78EventResult
) => {
  const { transactionInfo, executionResult } = await cep78
    .getTransactionResult(eventResult.transactionInfo.transactionHash.toHex())
    .then((transactionResult: InfoGetTransactionResult) => ({
      transactionInfo: eventResult.transactionInfo,
      executionResult: transactionResult.executionInfo?.executionResult,
    }));

  console.info(
    `Contract ${eventType} transaction hash: ${transactionInfo.transactionHash.toHex()}`
  );

  if (executionResult) {
    if (executionResult?.errorMessage) {
      throw new Error(
        `Error during ${eventType}.\n${executionResult?.errorMessage.toString()}`
      );
    } else {
      console.info(
        `Contract ${eventType} cost consumed: ${executionResult?.consumed}`
      );
    }
  }
  cep78.removeListenersForEvent(CEP78_EVENTS[eventType]);
};

usage()
  .then(() => {
    console.info('Events usage completed.');
  })
  .catch((error) => {
    console.error('Usage failed:', error);
  });
