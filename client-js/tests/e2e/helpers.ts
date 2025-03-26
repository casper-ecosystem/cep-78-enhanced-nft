import {
  PRIVATE_KEY_FAUCET,
  PRIVATE_KEY_USER_1,
  PRIVATE_KEY_USER_2,
} from '../../config';
import {
  EVENTS_MODE,
  CEP78Client,
  METADATA_MUTABILITY,
  HOLDER_MODE,
  IDENTIFIER_MODE,
  NFT_METADATA_KIND,
  OWNERSHIP_MODE,
  TransactionResult,
  ApproveArgs,
} from '../../src';
import { getSigningKey } from '../utils';

// Ensure required private keys are set
const requiredKeys = {
  faucet: PRIVATE_KEY_FAUCET,
  ali: PRIVATE_KEY_USER_1,
  bob: PRIVATE_KEY_USER_2,
};
Object.entries(requiredKeys).forEach(([key, value]) => {
  if (!value) {
    throw new Error(
      `${key.toUpperCase()}_SECRET_KEY environment variable is not set.`
    );
  }
});

// Helper function to generate a random token hash
export const generateTokenHash = () =>
  `tokenHash-${Math.floor(Math.random() * 1000000)}`;

// Signing keys for different users
export const owner = getSigningKey(PRIVATE_KEY_FAUCET!),
  ali = getSigningKey(PRIVATE_KEY_USER_1!),
  bob = getSigningKey(PRIVATE_KEY_USER_2!);

// Collection configuration
export const collectionConfig = {
  collectionSymbol: 'CEP78',
  totalTokenSupply: String(1000),
  eventsMode: EVENTS_MODE.CES,
  holderMode: HOLDER_MODE.Mixed,
  identifierMode: IDENTIFIER_MODE.Hash,
  ownershipMode: OWNERSHIP_MODE.Transferable,
  nftMetadataKind: NFT_METADATA_KIND.CustomValidated,
  jsonSchema: {
    properties: {
      ucid: { name: 'ucid', description: '', required: true },
      ipfs_cid: { name: 'ipfs_cid', description: '', required: true },
      color: { name: 'color', description: '', required: false },
    },
  },
  metadataMutability: METADATA_MUTABILITY.Immutable,
  paymentAmount: String(600_000_000_000),
};

/**
 * Installs the NFT contract.
 */
export const install = async (
  client: CEP78Client,
  collectionName: string,
  additionalCollectionConfig?: any
): Promise<TransactionResult> => {
  return client.install({
    params: {
      sender: owner.publicKey,
      paymentAmount: collectionConfig.paymentAmount,
      signingKeys: [owner],
    },
    args: {
      collectionName,
      ...collectionConfig,
      ...additionalCollectionConfig,
    },
    waitForTransactionProcessed: true,
  });
};

/**
 * Mints an NFT with optional dynamic owner and token metadata.
 */
export const mint = async (
  client: CEP78Client,
  waitForTransactionProcessed = true,
  tokenHash = generateTokenHash(),
  tokenOwner = owner,
  sender = owner
): Promise<TransactionResult> => {
  return client.mint({
    params: {
      sender: sender.publicKey,
      paymentAmount: String(5_000_000_000),
      signingKeys: [sender],
    },
    args: {
      tokenOwner: tokenOwner.publicKey,
      tokenMetaData: {
        ucid: tokenHash,
        ipfs_cid: 'QmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR',
        color: 'Blue',
      },
      tokenHash,
    },
    waitForTransactionProcessed,
  });
};

/**
 * Approves a user as an operator for a specific token.
 */
export const approve = async (
  client: CEP78Client,
  tokenHash?: string,
  sender = ali,
  operator = bob
): Promise<TransactionResult> => {
  const approveArgs: ApproveArgs = {
    operator: operator.publicKey,
    tokenHash,
  };
  return client.approve({
    params: {
      sender: sender.publicKey,
      paymentAmount: String(5_000_000_000),
      signingKeys: [sender],
    },
    args: approveArgs,
    waitForTransactionProcessed: true,
  });
};
