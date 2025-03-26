import {
  AccountHash,
  AddressableEntityHash,
  ContractHash,
  ContractPackageHash,
  ExecutionResult,
  PrivateKey,
  PublicKey,
  PutTransactionResult,
} from 'casper-js-sdk';

export enum EVENTS_MODE {
  NoEvents = 0,
  CEP47 = 1,
  CES = 2,
  Native = 3,
  NativeBytes = 4,
}

export type InstallArgs = {
  collectionName: string;
  collectionSymbol: string;
  totalTokenSupply: string;
  eventsMode?: EVENTS_MODE;
  ownershipMode: OWNERSHIP_MODE;
  nftKind?: NFT_KIND;
  jsonSchema?: JSONSchemaObject;
  nftMetadataKind: NFT_METADATA_KIND;
  identifierMode: IDENTIFIER_MODE;
  metadataMutability: METADATA_MUTABILITY;
  allowMinting?: boolean;
  mintingMode?: MINTING_MODE;
  holderMode?: HOLDER_MODE;
  burnMode?: BURN_MODE;
  operatorBurnMode?: boolean;
  ownerReverseLookupMode?: OWNER_REVERSE_LOOKUP_MODE;
  packageOperatorMode?: boolean;
  aclWhitelist?: Entity[];
  aclPackageMode?: boolean;
  whitelistMode?: WHITELIST_MODE;
  namedKeyConventionMode?: NAMED_KEY_CONVENTION_MODE;
  accessKeyName?: string;
  hashKeyName?: string;
  transferFilterContract?: ContractHash;
};

export type UpgradeArgs = {
  collectionName: string;
  totalTokenSupply?: string;
  eventsMode?: EVENTS_MODE;
  aclPackageMode?: boolean;
  packageOperatorMode?: boolean;
  operatorBurnMode?: boolean;
};

export enum NAMED_KEY_CONVENTION_MODE {
  DerivedFromCollectionName,
  V1_0Standard,
  V1_0Custom,
}

export enum OWNERSHIP_MODE {
  Minter,
  Assigned,
  Transferable,
}

export enum NFT_KIND {
  Physical,
  Digital,
  Virtual,
}

export enum HOLDER_MODE {
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

export enum IDENTIFIER_MODE {
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

export type Entity =
  | PublicKey
  | AccountHash
  | ContractHash
  | ContractPackageHash
  | AddressableEntityHash;

export interface TokenOwnerArg {
  tokenOwner: Entity;
}

export interface MintArgs extends TokenOwnerArg {
  tokenMetaData: Record<string, string>;
  tokenHash?: string;
}

export interface RegisterArgs extends TokenOwnerArg {}

export interface TokenArgs {
  tokenId?: string;
  tokenHash?: string;
}

export type tokenOwnerArg = {
  tokenOwner: Entity;
};

export type OperatorArg = {
  operator: Entity;
};

export type OperatorArgs = tokenOwnerArg & OperatorArg;

export type BurnArgs = TokenArgs;

export type TransferArgs = { target: Entity; source: Entity } & TokenArgs;

export type TokenMetadataArgs = {
  tokenMetaData: Record<string, string>;
} & TokenArgs;

export type BalanceOfArgs = { tokenOwner: Entity; keyName: string };

export type GetApprovedArgs = { keyName: string } & TokenArgs;

export type OwnerOfArgs = { keyName: string } & TokenArgs;

export type ApproveArgs = { operator: Entity } & TokenArgs;

export type RevokeArgs = { operator: Entity } & TokenArgs;

export type SetApprovallForAllArgs = {
  approveAll: boolean;
} & OperatorArg;

export type IsApprovedForAllArgs = {
  keyName: string;
} & OperatorArgs;

export type SetVariablesArgs = {
  allowMinting?: boolean;
  aclWhitelist?: Entity[];
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

export interface RegisterParams extends BaseParams {
  args: RegisterArgs;
}

export interface RevokeParams extends BaseParams {
  args: RevokeArgs;
}

export interface SetApprovallForAllParams extends BaseParams {
  args: SetApprovallForAllArgs;
}

export interface StoreBalanceOfParams extends BaseParams {
  args: BalanceOfArgs;
}

export type BalanceOfParams = Entity | StoreBalanceOfParams;

export interface StoreOwnerOfParams extends BaseParams {
  args: OwnerOfArgs;
}

export type OwnerOfParams = string | StoreOwnerOfParams;

export interface StoreGetApprovedParams extends BaseParams {
  args: GetApprovedArgs;
}

export type GetApprovedParams = string | StoreOwnerOfParams;

export interface StoreIsApprovedForAlldParams extends BaseParams {
  args: IsApprovedForAllArgs;
}

export type IsApprovedForAlldParams =
  | OperatorArgs
  | StoreIsApprovedForAlldParams;

export interface SetVariablesParams extends BaseParams {
  args: SetVariablesArgs;
}

export type isAclWhitelistedParams = Entity;

export interface updatedReceiptsParams extends BaseParams {}
