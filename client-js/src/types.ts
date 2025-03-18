import {
  ExecutionResult,
  PrivateKey,
  PublicKey,
  PutTransactionResult,
} from 'casper-js-sdk';

export enum EVENTS_MODE {
  NoEvents = 0,
  CES = 1,
  Native = 2,
  NativeBytes = 3,
}

export type InstallArgs = {
  collectionName: string;
  collectionSymbol: string;
  totalTokenSupply: string;
  eventsMode?: EVENTS_MODE;
  ownershipMode: NFT_OWNERSHIP_MODE;
  nftKind?: NFT_KIND;
  jsonSchema?: JSONSchemaObject;
  nftMetadataKind: NFT_METADATA_KIND;
  identifierMode: NFT_IDENTIFIER_MODE;
  metadataMutability: METADATA_MUTABILITY;
  allowMinting?: boolean;
  mintingMode?: MINTING_MODE;
  holderMode?: NFT_HOLDER_MODE;
  burnMode?: BURN_MODE;
  operatorBurnMode?: boolean;
  ownerReverseLookupMode?: OWNER_REVERSE_LOOKUP_MODE;
  packageOperatorMode?: boolean;
  aclWhitelist?: PublicKey[];
  aclPackageMode?: boolean;
  whitelistMode?: WHITELIST_MODE;
  namedKeyConventionMode?: NAMED_KEY_CONVENTION_MODE;
  accessKeyName?: string;
  hashKeyName?: string;
};

export type UpgradeArgs = { collectionName: string; eventsMode?: EVENTS_MODE };

export enum NAMED_KEY_CONVENTION_MODE {
  DerivedFromCollectionName,
  V1_0Standard,
  V1_0Custom,
}

export enum NFT_OWNERSHIP_MODE {
  Minter,
  Assigned,
  Transferable,
}

export enum NFT_KIND {
  Physical,
  Digital,
  Virtual,
}

export enum NFT_HOLDER_MODE {
  Accounts,
  Contracts,
  Mixed,
}

export enum NFT_METADATA_KIND {
  CEP78,
  NFT721,
  Raw,
  CustomValidated,
}

export enum NFT_IDENTIFIER_MODE {
  Ordinal,
  Hash,
}

export enum METADATA_MUTABILITY {
  Immutable,
  Mutable,
}

export enum MINTING_MODE {
  Installer,
  Public,
  Acl,
}

export enum BURN_MODE {
  Burnable,
  NonBurnable,
}

export enum WHITELIST_MODE {
  Unlocked,
  Locked,
}

export enum OWNER_REVERSE_LOOKUP_MODE {
  NoLookup,
  Complete,
  TransfersOnly,
}

export type TransactionParams = {
  sender: PublicKey;
  paymentAmount: string;
  wasm?: Uint8Array;
  callSessionWasm?: boolean;
  signingKeys?: PrivateKey[];
  chainName?: string;
};

export type TransactionResult = {
  transactionInfo: PutTransactionResult;
  executionResult?: ExecutionResult;
};

export interface JSONSchemaEntry {
  name: string;
  description: string;
  required: boolean;
}

export interface JSONSchemaObject {
  properties: Record<string, JSONSchemaEntry>;
}

export interface RegisterArgs {
  tokenOwner: PublicKey;
}

export interface MintArgs {
  tokenOwner: PublicKey;
  tokenMetaData: Record<string, string>;
  tokenHash?: string;
  collectionName?: string;
}

export interface TokenArgs {
  tokenId?: string;
  tokenHash?: string;
}

export type BurnArgs = TokenArgs;

export type TransferArgs = { target: PublicKey; source: PublicKey } & TokenArgs;

export type TokenMetadataArgs = { tokenMetaData: Record<string, string> };

export type BalanceOfArgs = { tokenOwner: PublicKey; keyName?: string };

export type GetApprovedArgs = { keyName?: string } & TokenArgs;

export type OwnerOfArgs = GetApprovedArgs;

export type RegisterOwnerArgs = { tokenOwner: PublicKey };

export type ApproveArgs = { operator: PublicKey } & TokenArgs;

export type RevokeArgs = { operator: PublicKey } & TokenArgs;

export type SetApprovallForAllArgs = {
  tokenOwner: PublicKey;
  operator: PublicKey;
  approveAll: boolean;
};

export type IsApprovedForAllArgs = {
  tokenOwner: PublicKey;
  operator: PublicKey;
  keyName?: string;
};

export type SetVariablesArgs = {
  allowMinting?: boolean;
  aclWhitelist?: PublicKey[];
  aclPackageMode?: boolean;
  packageOperatorMode?: boolean;
  operatorBurnMode?: boolean;
};

interface BaseParams {
  params: TransactionParams;
  waitForTransactionProcessed?: boolean;
}

export interface InstallParams extends BaseParams {
  args: InstallArgs;
}

export interface TransferParams extends BaseParams {
  args: TransferArgs;
}

export interface UpgradeParams extends BaseParams {
  args: UpgradeArgs;
}

export interface MintParams extends BaseParams {
  args: MintArgs;
}

export interface BurnParams extends BaseParams {
  args: BurnArgs;
}

export interface TokenMetadataParams extends BaseParams {
  args: TokenMetadataArgs;
}

export interface ApproveParams extends BaseParams {
  args: ApproveArgs;
}

export interface RegisterOwnerParams extends BaseParams {
  args: RegisterOwnerArgs;
}

export interface RevokeParams extends BaseParams {
  args: RevokeArgs;
}

export interface SetApprovallForAllParams extends BaseParams {
  args: SetApprovallForAllArgs;
}

export interface balanceOfParams extends BaseParams {
  args: BalanceOfArgs;
}

export interface OwnerOfParams extends BaseParams {
  args: OwnerOfArgs;
}

export interface GetApprovedParams extends BaseParams {
  args: GetApprovedArgs;
}

export interface IsApprovedForAlldParams extends BaseParams {
  args: IsApprovedForAllArgs;
}

export interface SetVariablesParams extends BaseParams {
  args: SetVariablesArgs;
}

export interface getMetadataOfParams {
  args: TokenArgs;
}

export interface updatedReceiptsParams extends BaseParams {}
