import {
  CHAIN_NAME,
  PRIVATE_KEY_FAUCET,
  PRIVATE_KEY_USER_1,
  RPC_URL,
  SSE_URL,
} from '../config';
import {
  CEP78Client,
  EVENTS_MODE,
  type InstallArgs,
  type TransactionParams,
  type TransactionResult,
  NFT_METADATA_KIND,
  IDENTIFIER_MODE,
  METADATA_MUTABILITY,
  MINTING_MODE,
  OWNER_REVERSE_LOOKUP_MODE,
  OWNERSHIP_MODE,
  NFT_KIND,
  HOLDER_MODE,
  WHITELIST_MODE,
} from '../dist';
import {
  findKeyFromAccountNamedKeys,
  getAccountInfo,
  getSigningKey,
} from '../tests/utils';

if (!PRIVATE_KEY_FAUCET) {
  throw new Error('FAUCET_SECRET_KEY environment variable is not set.');
}
if (!PRIVATE_KEY_USER_1) {
  throw new Error('PRIVATE_KEY_USER_1 environment variable is not set.');
}

const collectionName = 'TEST_CEP78',
  collectionSymbol = 'CEP78',
  totalTokenSupply = String(1000),
  eventsMode = EVENTS_MODE.CES,
  sender = getSigningKey(PRIVATE_KEY_FAUCET),
  ali = getSigningKey(PRIVATE_KEY_USER_1),
  holderMode = HOLDER_MODE.Mixed,
  mintingMode = MINTING_MODE.Acl,
  aclWhitelist = [sender.publicKey, ali.publicKey],
  whitelistMode = WHITELIST_MODE.Unlocked,
  ownershipMode = OWNERSHIP_MODE.Transferable,
  nftKind = NFT_KIND.Virtual,
  nftMetadataKind = NFT_METADATA_KIND.CustomValidated,
  jsonSchema = {
    properties: {
      ucid: { name: 'ucid', description: '', required: true },
      ipfs_cid: { name: 'ipfs_cid', description: '', required: true },
      color: { name: 'color', description: '', required: false },
    },
  },
  identifierMode = IDENTIFIER_MODE.Hash,
  metadataMutability = METADATA_MUTABILITY.Immutable,
  ownerReverseLookupMode = OWNER_REVERSE_LOOKUP_MODE.NoLookup,
  paymentAmount = String(600_000_000_000),
  waitForTransactionProcessed = true;

const install = async () => {
  const cep78 = new CEP78Client(RPC_URL, SSE_URL, CHAIN_NAME);

  const params: TransactionParams = {
    sender: sender.publicKey,
    paymentAmount,
    signingKeys: [sender],
  };

  const args: InstallArgs = {
    collectionName,
    collectionSymbol,
    totalTokenSupply,
    nftKind,
    holderMode,
    eventsMode,
    ownershipMode,
    jsonSchema,
    nftMetadataKind,
    identifierMode,
    metadataMutability,
    mintingMode,
    ownerReverseLookupMode,
    whitelistMode,
    aclWhitelist,

    // A transfer Filter Contract(cep-82) can be set that way
    // transferFilterContract: ContractHash.newContract(
    //   'hash-5eab221b01c32145051f47fa8c778b5a9ac5e01502d48dd13e5caa4973106906'
    // ),
  };

  const transactionResult: TransactionResult = await cep78.install({
    params,
    args,
    waitForTransactionProcessed,
  });

  if (!transactionResult.transactionInfo.transactionHash) {
    throw Error('Invalid transaction hash');
  }
  return transactionResult;
};

install()
  .then(async (transactionResult) => {
    const { transactionInfo, executionResult } = transactionResult;
    console.info(
      `Contract installation transaction hash: ${transactionInfo.transactionHash.toHex()}`
    );

    if (executionResult) {
      if (executionResult?.errorMessage) {
        throw new Error(
          `Error during installation.\n${executionResult?.errorMessage.toString()}`
        );
      } else {
        console.info(
          `Contract installation cost consumed: ${executionResult?.consumed}`
        );
      }
    }

    const account = await getAccountInfo(RPC_URL, sender.publicKey),
      contractHash = findKeyFromAccountNamedKeys(
        account,
        `cep78_contract_hash_${collectionName}`
      ),
      contractPackageHash = findKeyFromAccountNamedKeys(
        account,
        `cep78_contract_package_${collectionName}`
      );

    console.info(`Contract Hash: ${contractHash}`);
    console.info(`Contract Package Hash: ${contractPackageHash}`);
  })
  .catch((error) => {
    console.error(error);
  });
