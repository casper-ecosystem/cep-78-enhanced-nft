import {
  PRIVATE_KEY_FAUCET,
  PRIVATE_KEY_USER_1,
  PRIVATE_KEY_USER_2,
} from '../../config';
import {
  EVENTS_MODE,
  CEP78Client,
  METADATA_MUTABILITY,
  NFT_HOLDER_MODE,
  NFT_IDENTIFIER_MODE,
  NFT_METADATA_KIND,
  NFT_OWNERSHIP_MODE,
  TransactionParams,
  TransactionResult,
  ApproveArgs,
} from '../../src';
import { getSigningKey } from '../utils';

if (!PRIVATE_KEY_FAUCET) {
  throw new Error('FAUCET_SECRET_KEY environment variable is not set.');
}
if (!PRIVATE_KEY_USER_1) {
  throw new Error('PRIVATE_KEY_USER_1 environment variable is not set.');
}
if (!PRIVATE_KEY_USER_2) {
  throw new Error('PRIVATE_KEY_USER_2 environment variable is not set.');
}

export const collectionSymbol = 'CEP78';
export const totalTokenSupply = String(1000);
export const eventsMode = EVENTS_MODE.CES;
export const holderMode = NFT_HOLDER_MODE.Mixed;
export const identifierMode = NFT_IDENTIFIER_MODE.Hash;
export const ownershipMode = NFT_OWNERSHIP_MODE.Transferable;
export const nftMetadataKind = NFT_METADATA_KIND.CustomValidated;
export const jsonSchema = {
  properties: {
    ucid: { name: 'ucid', description: '', required: true },
    ipfs_cid: { name: 'ipfs_cid', description: '', required: true },
    color: { name: 'color', description: '', required: false },
  },
};
export const defaultTokenHash = 'tokenHash';
export const metadataMutability = METADATA_MUTABILITY.Immutable;
export const paymentAmount = String(600_000_000_000);
export const owner = getSigningKey(PRIVATE_KEY_FAUCET);
export const ali = getSigningKey(PRIVATE_KEY_USER_1);
export const bob = getSigningKey(PRIVATE_KEY_USER_2);

export const install = async (
  client: CEP78Client,
  collectionName: string
): Promise<TransactionResult> => {
  const params: TransactionParams = {
      sender: owner.publicKey,
      paymentAmount,
      signingKeys: [owner],
    },
    args = {
      collectionName,
      collectionSymbol,
      totalTokenSupply,
      eventsMode,
      holderMode,
      identifierMode,
      ownershipMode,
      nftMetadataKind,
      jsonSchema,
      metadataMutability,
    };

  return client.install({
    params,
    args,
    waitForTransactionProcessed: true,
  });
};

export const mint = async (
  client: CEP78Client,
  tokenHash = defaultTokenHash,
  waitForTransactionProcessed: boolean = true
): Promise<TransactionResult> => {
  return client.mint({
    params: {
      sender: owner.publicKey,
      paymentAmount: String(5_000_000_000),
      signingKeys: [owner],
    },
    args: {
      tokenOwner: owner.publicKey,
      tokenMetaData: {
        ucid: 'custom_hash',
        ipfs_cid: 'QmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR',
        color: 'Blue',
      },
      tokenHash,
    },
    waitForTransactionProcessed,
  });
};

export const approve = async (
  client: CEP78Client,
  tokenHash = defaultTokenHash
): Promise<TransactionResult> => {
  const approveArgs: ApproveArgs = {
    operator: bob.publicKey,
    tokenHash,
  };

  const params: TransactionParams = {
    sender: ali.publicKey,
    paymentAmount: String(5_000_000_000),
    signingKeys: [ali],
  };

  return client.approve({
    params,
    args: approveArgs,
    waitForTransactionProcessed: true,
  });
};
