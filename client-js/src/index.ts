import {
  CLValue,
  CLPublicKey,
  CLKey,
  CLMap,
  RuntimeArgs,
  CasperClient,
  Contracts,
  Keys,
  CLKeyParameters,
  CLValueBuilder,
  CLValueParsers,
  CLTypeTag,
  CLU8,
} from "casper-js-sdk";
import { concat } from "@ethersproject/bytes";
import { Some } from "ts-results";

const { Contract, toCLMap, fromCLMap } = Contracts;

import {
  CEP78InstallArgs,
  BurnMode,
  NFTHolderMode,
  NFTIdentifierMode,
  MetadataMutability,
  NFTOwnershipMode,
  NFTMetadataKind,
  NFTKind,
} from "./types";

export {
  CEP78InstallArgs,
  NFTOwnershipMode,
  NFTKind,
  NFTHolderMode,
  NFTMetadataKind,
  NFTIdentifierMode,
  MetadataMutability,
  MintingMode,
  BurnMode,
  WhitelistMode,
  JSONSchemaEntry,
  JSONSchemaObject,
} from "./types";

export class CEP78Client {
  casperClient: CasperClient;
  contractClient: Contracts.Contract;

  constructor(public nodeAddress: string, public networkName: string) {
    this.casperClient = new CasperClient(nodeAddress);
    this.contractClient = new Contract(this.casperClient);
  }

  public install(
    wasm: Uint8Array,
    args: CEP78InstallArgs,
    paymentAmount: string,
    deploySender: CLPublicKey,
    keys?: Keys.AsymmetricKey[]
  ) {
    const runtimeArgs = RuntimeArgs.fromMap({
      collection_name: CLValueBuilder.string(args.collectionName),
      collection_symbol: CLValueBuilder.string(args.collectionSymbol),
      total_token_supply: CLValueBuilder.u64(args.totalTokenSupply),
      ownership_mode: CLValueBuilder.u8(args.ownershipMode),
      nft_kind: CLValueBuilder.u8(args.nftKind),
      json_schema: CLValueBuilder.string(JSON.stringify(args.jsonSchema)),
      nft_metadata_kind: CLValueBuilder.u8(args.nftMetadataKind),
      identifier_mode: CLValueBuilder.u8(args.identifierMode),
      metadata_mutability: CLValueBuilder.u8(args.metadataMutability),
    });

    if (args.mintingMode) {
      const value = CLValueBuilder.u8(args.mintingMode);
      runtimeArgs.insert("minting_mode", CLValueBuilder.option(Some(value)));
    }

    if (args.allowMinting) {
      const value = CLValueBuilder.bool(args.allowMinting);
      runtimeArgs.insert("allow_minting", CLValueBuilder.option(Some(value)));
    }

    if (args.whitelistMode) {
      const value = CLValueBuilder.u8(args.whitelistMode);
      runtimeArgs.insert("whitelist_mode", CLValueBuilder.option(Some(value)));
    }

    if (args.holderMode) {
      const value = CLValueBuilder.u8(args.holderMode);
      runtimeArgs.insert("holder_mode", CLValueBuilder.option(Some(value)));
    }

    // TODO: Implement contractWhitelist support.

    if (args.burnMode) {
      const value = CLValueBuilder.u8(args.burnMode);
      runtimeArgs.insert("burn_mode", CLValueBuilder.option(Some(value)));
    }

    return this.contractClient.install(
      wasm,
      runtimeArgs,
      paymentAmount,
      deploySender,
      this.networkName,
      keys || []
    );
  }

  public setContractHash(
    contractHash: string,
    contractPackageHash?: string,
    bootstrap?: boolean
  ) {
    this.contractClient.setContractHash(contractHash, contractPackageHash);

    if (bootstrap) {
      // TODO: Set all possible config options inside the client and validate every client call.
    }
  }

  public async collectionName() {
    return this.contractClient.queryContractData(["collection_name"]);
  }

  public async collectionSymbol() {
    return this.contractClient.queryContractData(["collection_symbol"]);
  }

  public async tokenTotalSupply() {
    return this.contractClient.queryContractData(["total_token_supply"]);
  }

  public async numOfMintedTokens() {
    return this.contractClient.queryContractData(["number_of_minted_tokens"]);
  }

  public async getContractWhitelist() {
    return this.contractClient.queryContractData(["contract_whitelist"]);
  }

  public async getAllowMintingConfig() {
    return this.contractClient.queryContractData(["allow_minting"]);
  }

  public async getBurnModeConfig() {
    const internalValue = await this.contractClient.queryContractData([
      "burn_mode",
    ]);
    const u8res = (internalValue as CLU8).toString();
    return BurnMode[parseInt(u8res, 10)];
  }

  public async getHolderModeConfig() {
    const internalValue = await this.contractClient.queryContractData([
      "holder_mode",
    ]);
    const u8res = (internalValue as CLU8).toString();
    return NFTHolderMode[parseInt(u8res, 10)];
  }

  public async getIdentifierModeConfig() {
    const internalValue = await this.contractClient.queryContractData([
      "identifier_mode",
    ]);
    const u8res = (internalValue as CLU8).toString();
    return NFTIdentifierMode[parseInt(u8res, 10)];
  }

  public async getMetadataMutabilityConfig() {
    const internalValue = await this.contractClient.queryContractData([
      "metadata_mutability",
    ]);
    const u8res = (internalValue as CLU8).toString();
    return MetadataMutability[parseInt(u8res, 10)];
  }

  public async getNFTKindConfig() {
    const internalValue = await this.contractClient.queryContractData([
      "nft_kind",
    ]);
    const u8res = (internalValue as CLU8).toString();
    return NFTKind[parseInt(u8res, 10)];
  }

  public async getMetadataKindConfig() {
    const internalValue = await this.contractClient.queryContractData([
      "nft_metadata_kind",
    ]);
    const u8res = (internalValue as CLU8).toString();
    return NFTMetadataKind[parseInt(u8res, 10)];
  }

  public async getOwnershipModeConfig() {
    const internalValue = await this.contractClient.queryContractData([
      "ownership_mode",
    ]);
    const u8res = (internalValue as CLU8).toString();
    return NFTOwnershipMode[parseInt(u8res, 10)];
  }

  public async getJSONSchemaConfig() {
    const internalValue = await this.contractClient.queryContractData([
      "json_schema",
    ]);
    return internalValue.toString();
  }

  public async mint(
    owner: CLKeyParameters,
    meta: Record<string, string>,
    paymentAmount: string,
    deploySender: CLPublicKey,
    keys?: Keys.AsymmetricKey[],
    wasm?: Uint8Array
  ) {
    // TODO: Add metadata validation

    const runtimeArgs = RuntimeArgs.fromMap({
      token_owner: CLValueBuilder.key(owner),
      token_meta_data: CLValueBuilder.string(JSON.stringify(meta))
    });

    return this.contractClient.callEntrypoint(
      'mint',
      runtimeArgs,
      deploySender,
      this.networkName,
      paymentAmount,
      keys
    );
  }

  public async burn(
    tokenId: string,
    paymentAmount: string,
    deploySender: CLPublicKey,
    keys?: Keys.AsymmetricKey[],
    wasm?: Uint8Array
  ) {

    const runtimeArgs = RuntimeArgs.fromMap({
      token_id: CLValueBuilder.u64(tokenId),
    });

    return this.contractClient.callEntrypoint(
      'burn',
      runtimeArgs,
      deploySender,
      this.networkName,
      paymentAmount,
      keys
    );
  }
}
