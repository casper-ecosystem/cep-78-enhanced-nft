import {
  PRIVATE_KEY_FAUCET,
  SSE_URL,
  PRIVATE_KEY_USER_1,
  PRIVATE_KEY_USER_2,
  CHAIN_NAME,
  RPC_URL,
} from '../config';
import { CEP78Client, OWNER_REVERSE_LOOKUP_MODE } from '../dist';
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

const name = 'TEST_CEP78',
  owner = getSigningKey(PRIVATE_KEY_FAUCET),
  ali = getSigningKey(PRIVATE_KEY_USER_1),
  bob = getSigningKey(PRIVATE_KEY_USER_2),
  waitForTransactionProcessed = true;

const usage = async () => {
  const accountInfo = await getAccountInfo(RPC_URL, owner.publicKey),
    contractHash = findKeyFromAccountNamedKeys(
      accountInfo,
      `cep78_contract_hash_${name}`
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

  console.info('Collection info: ', {
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

  const useSessionCode =
    ownerReverseLookupMode ===
    OWNER_REVERSE_LOOKUP_MODE[OWNER_REVERSE_LOOKUP_MODE.Complete];

  console.info('Mint');

  const mintDeploy = cep78.mint(
    {
      owner: owner.publicKey,
      meta: {
        color: 'Blue',
        size: 'Medium',
        material: 'Aluminum',
        condition: 'Used',
      },
      collectionName: 'my-collection',
    },
    { useSessionCode },
    '2000000000',
    FAUCET_KEYS.publicKey,
    [FAUCET_KEYS]
  );

  await runDeployFlow(mintDeploy);
};

usage()
  .then(() => {
    console.info('Usage completed successfully.');
  })
  .catch((error) => {
    console.error('Usage failed:', error);
  });
