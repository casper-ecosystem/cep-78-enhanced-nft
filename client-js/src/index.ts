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
  DeployUtil,
} from "casper-js-sdk";
import { concat } from "@ethersproject/bytes";
import { Some } from "ts-results";
import * as fs from "fs";

const { Contract, toCLMap, fromCLMap } = Contracts;

import {
  InstallArgs,
  MintArgs,
  BurnArgs,
  TransferArgs,
  BurnMode,
  NFTHolderMode,
  NFTIdentifierMode,
  MetadataMutability,
  NFTOwnershipMode,
  NFTMetadataKind,
  NFTKind,
} from "./types";

export {
  InstallArgs,
  MintArgs,
  BurnArgs,
  TransferArgs,
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

const convertHashStrToHashBuff = (hashStr: string) => {
  let hashHex = hashStr;
  if (hashStr.startsWith("hash-")) {
    hashHex = hashStr.slice(5);
  }
  return Buffer.from(hashHex, "hex");
};

// TODO: In future, when we want to support also
// browser version - we will need to polyfill this
// and move to some separate module.
export const getBinary = (pathToBinary: string) => {
  return new Uint8Array(fs.readFileSync(pathToBinary, null).buffer);
};

export class CEP78Client {
  casperClient: CasperClient;
  contractClient: Contracts.Contract;
  contractHashKey: CLKey;

  constructor(public nodeAddress: string, public networkName: string) {
    this.casperClient = new CasperClient(nodeAddress);
    this.contractClient = new Contract(this.casperClient);
  }

  public install(
    args: InstallArgs,
    paymentAmount: string,
    deploySender: CLPublicKey,
    keys?: Keys.AsymmetricKey[],
    wasm?: Uint8Array
  ) {
    const wasmToInstall =
      wasm || getBinary(`${__dirname}/../wasm/contract.wasm`);

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

    if (args.mintingMode !== undefined) {
      runtimeArgs.insert("minting_mode", CLValueBuilder.u8(args.mintingMode));
    }

    if (args.allowMinting !== undefined) {
      runtimeArgs.insert(
        "allow_minting",
        CLValueBuilder.bool(args.allowMinting)
      );
    }

    if (args.whitelistMode !== undefined) {
      runtimeArgs.insert(
        "whitelist_mode",
        CLValueBuilder.u8(args.whitelistMode)
      );
    }

    if (args.holderMode !== undefined) {
      runtimeArgs.insert("holder_mode", CLValueBuilder.u8(args.holderMode));
    }

    if (args.contractWhitelist !== undefined) {
      const list = args.contractWhitelist.map((hashStr) =>
        CLValueBuilder.key(
          CLValueBuilder.byteArray(convertHashStrToHashBuff(hashStr))
        )
      );
      runtimeArgs.insert("holder_mode", CLValueBuilder.list(list));
    }

    if (args.burnMode !== undefined) {
      const value = CLValueBuilder.u8(args.burnMode);
      runtimeArgs.insert("burn_mode", CLValueBuilder.option(Some(value)));
    }

    return this.contractClient.install(
      wasmToInstall,
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
    this.contractHashKey = CLValueBuilder.key(
      CLValueBuilder.byteArray(convertHashStrToHashBuff(contractHash))
    );

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
    args: MintArgs,
    paymentAmount: string,
    deploySender: CLPublicKey,
    keys?: Keys.AsymmetricKey[],
    wasm?: Uint8Array
  ) {
    const wasmToCall = wasm || getBinary(`${__dirname}/../wasm/mint_call.wasm`);

    const runtimeArgs = RuntimeArgs.fromMap({
      nft_contract_hash: this.contractHashKey,
      token_owner: CLValueBuilder.key(args.owner),
      token_meta_data: CLValueBuilder.string(JSON.stringify(args.meta)),
    });

    const preparedDeploy = this.contractClient.install(
      wasmToCall,
      runtimeArgs,
      paymentAmount,
      deploySender,
      this.networkName,
      keys
    );

    return preparedDeploy;
  }

  public async burn(
    args: BurnArgs,
    paymentAmount: string,
    deploySender: CLPublicKey,
    keys?: Keys.AsymmetricKey[]
  ) {

    const runtimeArgs = RuntimeArgs.fromMap({});

    if (args.tokenId !== undefined) {
      runtimeArgs.insert("token_id", CLValueBuilder.u8(args.tokenId));
    }

    if (args.tokenHash !== undefined) {
      runtimeArgs.insert("token_hash", CLValueBuilder.string(args.tokenHash));
    }

    const preparedDeploy = this.contractClient.callEntrypoint(
      "burn",
      runtimeArgs,
      deploySender,
      this.networkName,
      paymentAmount,
      keys
    );

    return preparedDeploy;
  }

  public async transfer(
    args: TransferArgs,
    paymentAmount: string,
    deploySender: CLPublicKey,
    keys?: Keys.AsymmetricKey[],
    wasm?: Uint8Array
  ) {
    const wasmToCall = wasm || getBinary(`${__dirname}/../wasm/transfer_call.wasm`);

    const runtimeArgs = RuntimeArgs.fromMap({
      nft_contract_hash: this.contractHashKey,
      target_key: CLValueBuilder.key(args.target),
      source_key: CLValueBuilder.key(args.source),
      is_hash_identifier_mode: CLValueBuilder.bool(args.isHashIdentifierMode)
    });

    if (args.tokenId) {
      runtimeArgs.insert("is_hash_identifier_mode", CLValueBuilder.bool(false));
      runtimeArgs.insert("token_id", CLValueBuilder.u8(args.tokenId));
    }

    if (args.tokenHash) {
      runtimeArgs.insert("is_hash_identifier_mode", CLValueBuilder.bool(true));
      runtimeArgs.insert("token_id", CLValueBuilder.u8(args.tokenHash));
    }

    const preparedDeploy = this.contractClient.install(
      wasmToCall,
      runtimeArgs,
      paymentAmount,
      deploySender,
      this.networkName,
      keys
    );

    return preparedDeploy;
  }



}
