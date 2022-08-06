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
} from "casper-js-sdk";
import { concat } from "@ethersproject/bytes";
import { Some } from "ts-results";

const { Contract, toCLMap, fromCLMap } = Contracts;

import { CEP78InstallArgs } from "./types";

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
      runtimeArgs.insert('minting_mode', CLValueBuilder.option(Some(value)));
    }

    if (args.allowMinting) {
      const value = CLValueBuilder.bool(args.allowMinting);
      runtimeArgs.insert('allow_minting', CLValueBuilder.option(Some(value)));
    }

    if (args.whitelistMode) {
      const value = CLValueBuilder.u8(args.whitelistMode);
      runtimeArgs.insert('whitelist_mode', CLValueBuilder.option(Some(value)));
    }

    if (args.holderMode) {
      const value = CLValueBuilder.u8(args.holderMode);
      runtimeArgs.insert('holder_mode', CLValueBuilder.option(Some(value)));
    }

    // TODO: Implement contractWhitelist support.
    
    if (args.burnMode) {
      const value = CLValueBuilder.u8(args.burnMode);
      runtimeArgs.insert('burn_mode', CLValueBuilder.option(Some(value)));
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
}
