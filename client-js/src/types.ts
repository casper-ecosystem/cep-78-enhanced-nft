export enum NFTOwnershipMode {
  Minter,
  Assigned,
  Transferable
}

export enum NFTKind {
  Physical,
  Digital,
  Virtual 
}

export enum NFTHolderMode {
  Accounts,
  Contracts,
  Mixed
}

export enum NFTMetadataKind {
  CEP78,
  NFT721,
  Raw,
  CustomValidated
}

export enum NFTIdentifierMode {
  Ordinal,
  Hash
}

export enum MetadataMutability {
  Immutable,
  Mutable
}

export enum MintingMode {
  Installer,
  Public
}

export enum BurnMode {
  Burnable,
  NonBurnable
}

export enum WhitelistMode {
  Unlocked,
  Locked
}

export interface JSONSchemaEntry {
  name: string,
  description: string,
  required: boolean
}

export interface JSONSchemaObject { 
  properties: Record<string, JSONSchemaEntry>
}

export interface CEP78InstallArgs {
  collectionName: string,
  collectionSymbol: string,
  totalTokenSupply: string,
  ownershipMode: NFTOwnershipMode,
  nftKind: NFTKind,
  jsonSchema: JSONSchemaObject,
  nftMetadataKind: NFTMetadataKind,
  identifierMode: NFTIdentifierMode,
  metadataMutability: MetadataMutability,

  mintingMode?: MintingMode,
  allowMinting?: boolean,
  whitelistMode?: WhitelistMode,
  holderMode?: NFTHolderMode,
  contractWhitelist?: string[],
  burnMode?: BurnMode
};

