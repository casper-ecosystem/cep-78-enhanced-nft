import { CLKeyParameters } from "casper-js-sdk";

export enum NFTOwnershipMode {
  Minter,
  Assigned,
  Transferable,
}

export enum NFTKind {
  Physical,
  Digital,
  Virtual,
}

export enum NFTHolderMode {
  Accounts,
  Contracts,
  Mixed,
}

export enum NFTMetadataKind {
  CEP78,
  NFT721,
  Raw,
  CustomValidated,
}

export enum NFTIdentifierMode {
  Ordinal,
  Hash,
}

export enum MetadataMutability {
  Immutable,
  Mutable,
}

export enum MintingMode {
  Installer,
  Public,
}

export enum BurnMode {
  Burnable,
  NonBurnable,
}

export enum WhitelistMode {
  Unlocked,
  Locked,
}

export interface JSONSchemaEntry {
  name: string;
  description: string;
  required: boolean;
}

export interface JSONSchemaObject {
  properties: Record<string, JSONSchemaEntry>;
}

export type ConfigurableVariables = {
  allowMinting?: boolean;
  contractWhitelist?: string[];
};

export type InstallArgs = {
  collectionName: string;
  collectionSymbol: string;
  totalTokenSupply: string;
  ownershipMode: NFTOwnershipMode;
  nftKind: NFTKind;
  jsonSchema: JSONSchemaObject;
  nftMetadataKind: NFTMetadataKind;
  identifierMode: NFTIdentifierMode;
  metadataMutability: MetadataMutability;
  mintingMode?: MintingMode;
  whitelistMode?: WhitelistMode;
  holderMode?: NFTHolderMode;
  burnMode?: BurnMode;
} & ConfigurableVariables;

export interface MintArgs {
  owner: CLKeyParameters;
  meta: Record<string, string>;
}

export interface TokenArgs {
  tokenId?: string;
  tokenHash?: string;
}

export type BurnArgs = TokenArgs;

export type TransferArgs = {
  target: CLKeyParameters;
  source: CLKeyParameters;
} & TokenArgs;

export type TokenMetadataArgs = {
  tokenMetaData: string;
};

export type StoreBalanceOfArgs = {
  tokenOwner: CLKeyParameters;
  keyName: string;
};

export type StoreApprovedArgs = {
  keyName: string;
} & TokenArgs;

export type StoreOwnerOfArgs = StoreApprovedArgs;
