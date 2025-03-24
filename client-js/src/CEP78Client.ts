import { blake2b } from '@noble/hashes/blake2b';
import { bytesToHex } from '@noble/hashes/utils';
import {
  Args as RuntimeArgs,
  CLTypeKey,
  CLValue,
  ContractHash,
  ContractPackageHash,
  Key,
  ParamDictionaryIdentifier,
  ParamDictionaryIdentifierContractNamedKey,
  SessionBuilder,
  PublicKey,
} from 'casper-js-sdk';
import Client from './client';
import {
  type SetApprovallForAllParams,
  type ApproveParams,
  type BalanceOfParams,
  BURN_MODE,
  type BurnParams,
  EVENTS_MODE,
  type GetApprovedParams,
  METADATA_MUTABILITY,
  type MintParams,
  NAMED_KEY_CONVENTION_MODE,
  NFT_HOLDER_MODE,
  NFT_IDENTIFIER_MODE,
  NFT_KIND,
  NFT_METADATA_KIND,
  NFT_OWNERSHIP_MODE,
  OWNER_REVERSE_LOOKUP_MODE,
  type OwnerOfParams,
  type TokenMetadataParams,
  type UpgradeParams,
  WHITELIST_MODE,
  type InstallParams,
  type TransactionResult,
  type TransferParams,
  type IsApprovedForAlldParams,
  updatedReceiptsParams,
  RegisterParams,
  SetVariablesParams,
  StoreOwnerOfParams,
  StoreBalanceOfParams,
  MINTING_MODE,
  OperatorArgs,
  Entity,
  BalanceOfArgs,
  isAclWhitelistedParams,
} from './types';
import BalanceOfWASM from './wasm/balance_of_session';
import ContractWASM from './wasm/cep78';
import GetApprovedWASM from './wasm/get_approved_session';
import isApprovedForAllWASM from './wasm/is_approved_for_all_session';
import MintWASM from './wasm/mint_session';
import GetOwnerOfWASM from './wasm/owner_of_session';
import TransferWASM from './wasm/transfer_session';
import UpdatedReceiptsWASM from './wasm/updated_receipts';

const prefixRegex = /^.*-/;

/**
 * CEP78Client extends the base `Client` class to provide specific functionality
 * for interacting with CEP-78 token contracts on the Casper blockchain.
 */
export default class CEP78Client extends Client {
  /**
   * Initializes a new CEP78Client instance.
   *
   * @param rpcUrl - The RPC URL of the Casper network.
   * @param ssUrl - (Optional) The SSE URL for event streaming.
   * @param chainName - (Optional) The name of the blockchain network.
   */
  constructor(rpcUrl: string, ssUrl?: string, chainName?: string) {
    super(rpcUrl, ssUrl, chainName);
  }

  /**
   * Sets the contract hash and optionally the contract package hash.
   *
   * This method removes prefixes from the provided contract hash and package hash
   * before converting them into the appropriate `ContractHash` and `ContractPackageHash` objects.
   *
   * @param contractHash - The contract hash as a string or `ContractHash` instance.
   * @param contractPackageHash - (Optional) The contract package hash as a string or `ContractPackageHash` instance.
   * @returns The updated `CEP78Client` instance.
   * @throws `Error` if the contract hash is not provided or invalid.
   */
  public setContractHash(
    contractHash: string | ContractHash,
    contractPackageHash?: string | ContractPackageHash
  ): CEP78Client {
    const removePrefix = (str?: string) =>
      str ? str.replace(prefixRegex, '') : '';

    const hexContractHash =
        typeof contractHash === 'string' ? removePrefix(contractHash) : '',
      hexContractPackageHash =
        typeof contractPackageHash === 'string'
          ? removePrefix(contractPackageHash)
          : '',
      newContractHash = hexContractHash
        ? ContractHash.newContract(hexContractHash)
        : undefined,
      newContractPackageHash = hexContractPackageHash
        ? ContractPackageHash.newContractPackage(hexContractPackageHash)
        : undefined;

    if (!newContractHash) {
      throw new Error('Contract hash must be provided.');
    }
    return super.setContractHash(
      newContractHash,
      newContractPackageHash
    ) as unknown as CEP78Client;
  }

  /**
   * Starts the SSE event stream to listen for contract-related events.
   *
   * This method enables real-time event listening for the contract by calling
   * the parent `startEventStream` method.
   *
   * @param sseUrl - (Optional) The SSE endpoint URL. If not provided, the previously set URL is used.
   * @returns The updated `CEP78Client` instance.
   */
  public startEventStream(sseUrl?: string): CEP78Client {
    return super.startEventStream(sseUrl) as unknown as CEP78Client;
  }

  /**
   * Stops the SSE event stream, preventing further event processing.
   *
   * This method ensures that the event stream is properly stopped and unsubscribed.
   *
   * @returns The updated `CEP78Client` instance.
   */
  public stopEventStream(): CEP78Client {
    return super.stopEventStream() as unknown as CEP78Client;
  }

  public async install(params: InstallParams): Promise<TransactionResult> {
    const {
      params: { wasm, paymentAmount, sender, chainName, signingKeys },
      args: {
        collectionName,
        collectionSymbol,
        totalTokenSupply,
        eventsMode,
        ownershipMode,
        nftKind,
        jsonSchema,
        nftMetadataKind,
        identifierMode,
        metadataMutability,
        allowMinting,
        mintingMode,
        holderMode,
        burnMode,
        operatorBurnMode,
        ownerReverseLookupMode,
        packageOperatorMode,
        aclWhitelist,
        aclPackageMode,
        whitelistMode,
        namedKeyConventionMode,
        accessKeyName,
        hashKeyName,
      },
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      collection_name: CLValue.newCLString(collectionName),
      collection_symbol: CLValue.newCLString(collectionSymbol),
      total_token_supply: CLValue.newCLUint64(totalTokenSupply),
      ownership_mode: CLValue.newCLUint8(ownershipMode),
      nft_metadata_kind: CLValue.newCLUint8(nftMetadataKind),
      identifier_mode: CLValue.newCLUint8(identifierMode),
      metadata_mutability: CLValue.newCLUint8(metadataMutability),
    });

    if (nftKind !== undefined) {
      runtimeArgs.insert('nft_kind', CLValue.newCLUint8(nftKind));
    }

    if (jsonSchema !== undefined) {
      runtimeArgs.insert(
        'json_schema',
        CLValue.newCLString(JSON.stringify(jsonSchema))
      );
    }

    if (mintingMode !== undefined) {
      runtimeArgs.insert('minting_mode', CLValue.newCLUint8(mintingMode));
    }

    if (allowMinting !== undefined) {
      runtimeArgs.insert('allow_minting', CLValue.newCLValueBool(allowMinting));
    }

    if (operatorBurnMode !== undefined) {
      runtimeArgs.insert(
        'operator_burn_mode',
        CLValue.newCLValueBool(operatorBurnMode)
      );
    }

    if (packageOperatorMode !== undefined) {
      runtimeArgs.insert(
        'package_operator_mode',
        CLValue.newCLValueBool(packageOperatorMode)
      );
    }

    if (whitelistMode !== undefined) {
      runtimeArgs.insert('whitelist_mode', CLValue.newCLUint8(whitelistMode));
    }

    if (holderMode !== undefined) {
      runtimeArgs.insert('holder_mode', CLValue.newCLUint8(holderMode));
    }

    if (aclPackageMode !== undefined) {
      runtimeArgs.insert(
        'acl_package_mode',
        CLValue.newCLValueBool(aclPackageMode)
      );
    }

    if (aclWhitelist !== undefined) {
      const list = CLValue.newCLList(
        CLTypeKey,
        aclWhitelist.map((key) =>
          CLValue.newCLKey(Key.newKey(key.accountHash().toPrefixedString()))
        )
      );
      runtimeArgs.insert('acl_whitelist', list);
    }

    if (burnMode !== undefined) {
      runtimeArgs.insert('burn_mode', CLValue.newCLUint8(burnMode));
    }

    if (ownerReverseLookupMode !== undefined) {
      runtimeArgs.insert(
        'owner_reverse_lookup_mode',
        CLValue.newCLUint8(ownerReverseLookupMode)
      );
    }

    if (namedKeyConventionMode !== undefined) {
      runtimeArgs.insert(
        'named_key_convention',
        CLValue.newCLUint8(namedKeyConventionMode)
      );
    }

    if (namedKeyConventionMode === NAMED_KEY_CONVENTION_MODE.V1_0Custom) {
      if (!accessKeyName || !hashKeyName) {
        throw new Error(
          "You need to provide 'accessKeyName' and 'hashKeyName' if you want to use NamedKeyConventionMode.V1_0Custom"
        );
      }
      runtimeArgs.insert('access_key_name', CLValue.newCLString(accessKeyName));
      runtimeArgs.insert('hash_key_name', CLValue.newCLString(hashKeyName));
    }

    if (eventsMode !== undefined) {
      runtimeArgs.insert('events_mode', CLValue.newCLUint8(eventsMode));
    }

    const wasmBytes = wasm || ContractWASM;

    if (!wasmBytes) {
      throw new Error('Wasm file is missing.');
    }
    const transaction = new SessionBuilder()
      .installOrUpgrade()
      .wasm(wasmBytes)
      .runtimeArgs(runtimeArgs)
      .payment(Number(paymentAmount))
      .from(sender)
      .chainName(chainName ? chainName : this.chainName || '')
      .build();

    if (signingKeys) {
      signingKeys.forEach((key) => transaction.sign(key));
    }
    try {
      const transactionInfo = await this.rpcClient.putTransaction(transaction);
      if (
        params.waitForTransactionProcessed &&
        transactionInfo.transactionHash
      ) {
        const transactionProcessedEvent =
          await this.waitForTransactionProcessed(
            transactionInfo.transactionHash.toString()
          );
        const executionResult =
          transactionProcessedEvent.transactionProcessedPayload.executionResult;
        if (executionResult?.errorMessage) {
          this.handleExecutionError(executionResult.errorMessage);
        }
        return { transactionInfo, executionResult };
      }
      return { transactionInfo };
    } catch (error) {
      throw new Error(`Error during installation runtime.\n${error}`);
    }
  }

  public async upgrade(params: UpgradeParams): Promise<TransactionResult> {
    const {
      params: { wasm, paymentAmount, sender, chainName, signingKeys },
      args: { collectionName, eventsMode },
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      collection_name: CLValue.newCLString(collectionName),
    });

    if (eventsMode !== undefined) {
      runtimeArgs.insert('events_mode', CLValue.newCLUint8(eventsMode));
    }

    const wasmBytes = wasm || ContractWASM;

    if (!wasmBytes) {
      throw new Error('Wasm file is missing.');
    }

    const transaction = new SessionBuilder()
      .installOrUpgrade()
      .wasm(wasmBytes)
      .runtimeArgs(runtimeArgs)
      .payment(Number(paymentAmount))
      .from(sender)
      .chainName(chainName ? chainName : this.chainName || '')
      .build();

    if (signingKeys) {
      signingKeys.forEach((key) => transaction.sign(key));
    }
    try {
      const transactionInfo = await this.rpcClient.putTransaction(transaction);
      if (
        params.waitForTransactionProcessed &&
        transactionInfo.transactionHash
      ) {
        const transactionProcessedEvent =
          await this.waitForTransactionProcessed(
            transactionInfo.transactionHash.toString()
          );
        const executionResult =
          transactionProcessedEvent.transactionProcessedPayload.executionResult;
        if (executionResult?.errorMessage) {
          this.handleExecutionError(executionResult.errorMessage);
        }
        return { transactionInfo, executionResult };
      }
      return { transactionInfo };
    } catch (error) {
      throw new Error(`Error during upgrade runtime.\n${error}`);
    }
  }

  public mint(params: MintParams, callSessionWasm = false) {
    if (!this.contractHash) {
      throw Error('Contract hash is not set.');
    }
    const {
      params: { wasm, paymentAmount, sender, chainName, signingKeys },
      args: { tokenOwner, tokenMetaData, tokenHash },
      waitForTransactionProcessed,
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      token_owner: CLValue.newCLKey(this.getPrefixedString(tokenOwner)),
      token_meta_data: CLValue.newCLString(JSON.stringify(tokenMetaData)),
    });

    if (tokenHash) {
      runtimeArgs.insert('token_hash', CLValue.newCLString(tokenHash));
    }

    if (callSessionWasm) {
      const wasmBytes = wasm || MintWASM;
      if (!wasmBytes) {
        throw new Error('Wasm file is missing.');
      }

      // ! TODO toPrefixedString() ?
      const key = `hash-${this.contractHash?.hash?.toHex()}`;

      runtimeArgs.insert(
        'nft_contract_hash',
        CLValue.newCLKey(Key.newKey(key))
      );

      return this.callSession(
        wasmBytes,
        runtimeArgs,
        paymentAmount,
        sender,
        signingKeys,
        chainName,
        waitForTransactionProcessed
      );
    }

    return this.callEntrypoint(
      'mint',
      runtimeArgs,
      paymentAmount,
      sender,
      signingKeys,
      chainName,
      waitForTransactionProcessed
    );
  }

  public burn(params: BurnParams) {
    const {
      params: { paymentAmount, sender, chainName, signingKeys },
      args: { tokenId, tokenHash },
      waitForTransactionProcessed,
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({});

    if (tokenId) {
      runtimeArgs.insert('token_id', CLValue.newCLUint64(tokenId));
    } else if (tokenHash) {
      runtimeArgs.insert('token_hash', CLValue.newCLString(tokenHash));
    }

    return this.callEntrypoint(
      'burn',
      runtimeArgs,
      paymentAmount,
      sender,
      signingKeys,
      chainName,
      waitForTransactionProcessed
    );
  }

  public async transfer(
    params: TransferParams,
    callSessionWasm = false
  ): Promise<TransactionResult> {
    const {
      params: { wasm, sender, paymentAmount, signingKeys, chainName },
      args: { target, source, tokenId, tokenHash },
      waitForTransactionProcessed,
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      target_key: CLValue.newCLKey(this.getPrefixedString(target)),
      source_key: CLValue.newCLKey(this.getPrefixedString(source)),
    });

    if (tokenId) {
      runtimeArgs.insert('token_id', CLValue.newCLUint64(tokenId));
    } else if (tokenHash) {
      runtimeArgs.insert('token_hash', CLValue.newCLString(tokenHash));
    }

    if (callSessionWasm) {
      const wasmBytes = wasm || TransferWASM;
      if (!wasmBytes) {
        throw new Error('Wasm file is missing.');
      }

      // ! TODO toPrefixedString() ?
      const key = `hash-${this.contractHash?.hash?.toHex()}`;

      runtimeArgs.insert(
        'nft_contract_hash',
        CLValue.newCLKey(Key.newKey(key))
      );

      return this.callSession(
        wasmBytes,
        runtimeArgs,
        paymentAmount,
        sender,
        signingKeys,
        chainName,
        waitForTransactionProcessed
      );
    }

    return this.callEntrypoint(
      'transfer',
      runtimeArgs,
      paymentAmount,
      sender,
      signingKeys,
      chainName,
      waitForTransactionProcessed
    );
  }

  public register(params: RegisterParams): Promise<TransactionResult> {
    const {
      params: { paymentAmount, sender, chainName, signingKeys },
      args: { tokenOwner },
      waitForTransactionProcessed,
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      token_owner: CLValue.newCLKey(this.getPrefixedString(tokenOwner)),
    });

    return this.callEntrypoint(
      'register_owner',
      runtimeArgs,
      paymentAmount,
      sender,
      signingKeys,
      chainName,
      waitForTransactionProcessed
    );
  }

  public approve(params: ApproveParams): Promise<TransactionResult> {
    const {
      params: { sender, paymentAmount, signingKeys, chainName },
      args: { operator, tokenId, tokenHash },
      waitForTransactionProcessed,
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      operator: CLValue.newCLKey(this.getPrefixedString(operator)),
    });

    if (tokenId) {
      runtimeArgs.insert('token_id', CLValue.newCLUint64(tokenId));
    } else if (tokenHash) {
      runtimeArgs.insert('token_hash', CLValue.newCLString(tokenHash));
    }

    return this.callEntrypoint(
      'approve',
      runtimeArgs,
      paymentAmount,
      sender,
      signingKeys,
      chainName,
      waitForTransactionProcessed
    );
  }

  public revoke(params: ApproveParams): Promise<TransactionResult> {
    const {
      params: { sender, paymentAmount, signingKeys, chainName },
      args: { operator, tokenId, tokenHash },
      waitForTransactionProcessed,
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      operator: CLValue.newCLKey(this.getPrefixedString(operator)),
    });

    if (tokenId) {
      runtimeArgs.insert('token_id', CLValue.newCLUint64(tokenId));
    } else if (tokenHash) {
      runtimeArgs.insert('token_hash', CLValue.newCLString(tokenHash));
    }

    return this.callEntrypoint(
      'revoke',
      runtimeArgs,
      paymentAmount,
      sender,
      signingKeys,
      chainName,
      waitForTransactionProcessed
    );
  }

  public setApprovalForAll(
    params: SetApprovallForAllParams
  ): Promise<TransactionResult> {
    const {
      params: { paymentAmount, sender, chainName, signingKeys },
      args: { operator, approveAll },
      waitForTransactionProcessed,
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      approve_all: CLValue.newCLValueBool(approveAll),
      operator: CLValue.newCLKey(this.getPrefixedString(operator)),
    });

    return this.callEntrypoint(
      'set_approval_for_all',
      runtimeArgs,
      paymentAmount,
      sender,
      signingKeys,
      chainName,
      waitForTransactionProcessed
    );
  }

  public setTokenMetadata(
    params: TokenMetadataParams
  ): Promise<TransactionResult> {
    const {
      params: { sender, paymentAmount, signingKeys, chainName },
      args: { tokenMetaData },
      waitForTransactionProcessed,
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      token_meta_data: CLValue.newCLString(JSON.stringify(tokenMetaData)),
    });

    return this.callEntrypoint(
      'set_token_metadata',
      runtimeArgs,
      paymentAmount,
      sender,
      signingKeys,
      chainName,
      waitForTransactionProcessed
    );
  }

  public async ownerOf(
    params: OwnerOfParams
  ): Promise<string | TransactionResult | undefined> {
    if (!this.contractHash) {
      throw new Error('Contract hash is not set.');
    }
    // ! TODO toPrefixedString() ?
    const key = `hash-${this.contractHash?.hash?.toHex()}`;

    if (typeof params === 'object') {
      const {
        params: { wasm, sender, paymentAmount, signingKeys, chainName },
        args: { tokenId, tokenHash, keyName },
        waitForTransactionProcessed,
      } = params as StoreOwnerOfParams;

      const runtimeArgs = this.addTokenIdentifierRuntimeArgs(
        RuntimeArgs.fromMap({}),
        tokenId,
        tokenHash
      );

      if (keyName) {
        const wasmBytes = wasm || GetOwnerOfWASM;
        if (!wasmBytes) {
          throw new Error('Wasm file is missing.');
        }

        runtimeArgs.insert(
          'nft_contract_hash',
          CLValue.newCLKey(Key.newKey(key))
        );
        runtimeArgs.insert('key_name', CLValue.newCLString(keyName));

        return this.callSession(
          wasmBytes,
          runtimeArgs,
          paymentAmount!,
          sender!,
          signingKeys,
          chainName,
          waitForTransactionProcessed
        );
      }
    }

    const tokenIdentifier = params as string;

    const dictionaryItemKey = tokenIdentifier;

    const contractNamedKey = new ParamDictionaryIdentifierContractNamedKey(
      key,
      'token_owners',
      dictionaryItemKey!
    );

    const identifier = new ParamDictionaryIdentifier(
      undefined,
      contractNamedKey,
      undefined,
      undefined
    );

    try {
      const stateGetDictionaryResult =
        await this.rpcClient.getDictionaryItemByIdentifier(null, identifier);
      return stateGetDictionaryResult.storedValue.clValue?.toString();
    } catch (error) {
      if (error instanceof Error && error.toString().includes('Query failed')) {
        console.warn(`No owner found for ${tokenIdentifier}`);
        return undefined;
      } else throw error;
    }
  }

  public async balanceOf(
    params: BalanceOfParams
  ): Promise<TransactionResult | string> {
    if (!this.contractHash) {
      throw Error('Contract hash is not set.');
    }

    // ! TODO toPrefixedString() ?
    const key = `hash-${this.contractHash?.hash?.toHex()}`;

    if (this.isStoreBalanceOfParams(params)) {
      const {
        params: { wasm, sender, paymentAmount, signingKeys, chainName },
        args: { tokenOwner, keyName },
        waitForTransactionProcessed,
      } = params as StoreBalanceOfParams;
      const runtimeArgs = RuntimeArgs.fromMap({
        token_owner: CLValue.newCLKey(this.getPrefixedString(tokenOwner)),
      });

      if (keyName) {
        const wasmBytes = wasm || BalanceOfWASM;
        if (!wasmBytes) {
          throw new Error('Wasm file is missing.');
        }

        runtimeArgs.insert(
          'nft_contract_hash',
          CLValue.newCLKey(Key.newKey(key))
        );
        runtimeArgs.insert('key_name', CLValue.newCLString(keyName));

        return this.callSession(
          wasmBytes,
          runtimeArgs,
          paymentAmount,
          sender,
          signingKeys,
          chainName,
          waitForTransactionProcessed
        );
      }
    }
    let tokenOwnerKey: Key = this.getPrefixedString(params as Entity);

    const dictionaryItemKey = tokenOwnerKey
      .toPrefixedString()
      .replace(prefixRegex, '');
    const contractNamedKey: ParamDictionaryIdentifierContractNamedKey =
      new ParamDictionaryIdentifierContractNamedKey(
        key,
        'balances',
        dictionaryItemKey
      );

    const identifier = new ParamDictionaryIdentifier(
      undefined,
      contractNamedKey,
      undefined,
      undefined
    );
    let balance = '0';
    try {
      balance =
        (
          await this.rpcClient.getDictionaryItemByIdentifier(null, identifier)
        ).storedValue.clValue?.toString() || balance;
    } catch (error) {
      if (error instanceof Error && error.toString().includes('Query failed')) {
        console.warn(
          `No balance found for ${tokenOwnerKey.toPrefixedString()}`
        );
      } else throw error;
    }
    return balance;
  }

  public async getApproved(
    params: GetApprovedParams
  ): Promise<string | TransactionResult | undefined> {
    if (!this.contractHash) {
      throw new Error('Contract hash is not set.');
    }
    // ! TODO toPrefixedString() ?
    const key = `hash-${this.contractHash?.hash?.toHex()}`;

    if (typeof params === 'object') {
      const {
        params: { wasm, sender, paymentAmount, signingKeys, chainName },
        args: { tokenId, tokenHash, keyName },
        waitForTransactionProcessed,
      } = params;

      if (keyName) {
        const wasmBytes = wasm || GetApprovedWASM;
        if (!wasmBytes) {
          throw new Error('Wasm file is missing.');
        }

        const runtimeArgs = RuntimeArgs.fromMap({
          nft_contract_hash: CLValue.newCLKey(Key.newKey(key)),
          key_name: CLValue.newCLString(keyName),
        });

        if (tokenId) {
          runtimeArgs.insert('token_id', CLValue.newCLUint64(tokenId));
        } else if (tokenHash) {
          runtimeArgs.insert('token_hash', CLValue.newCLString(tokenHash));
        }

        return this.callSession(
          wasmBytes,
          runtimeArgs,
          paymentAmount,
          sender,
          signingKeys,
          chainName,
          waitForTransactionProcessed
        );
      }
    }

    const tokenIdentifier = params as string;

    const dictionaryItemKey = tokenIdentifier;

    const contractNamedKey: ParamDictionaryIdentifierContractNamedKey =
      new ParamDictionaryIdentifierContractNamedKey(
        key,
        'approved',
        dictionaryItemKey!
      );

    const identifier = new ParamDictionaryIdentifier(
      undefined,
      contractNamedKey,
      undefined,
      undefined
    );

    try {
      return (
        await this.rpcClient.getDictionaryItemByIdentifier(null, identifier)
      ).storedValue.clValue?.toString();
    } catch (error) {
      if (error instanceof Error && error.toString().includes('Query failed')) {
        console.warn(`No approval found for ${tokenIdentifier}`);
        return '';
      } else throw error;
    }
  }

  public async isApprovedForAll(
    params: IsApprovedForAlldParams
  ): Promise<boolean | TransactionResult> {
    if (!this.contractHash) {
      throw new Error('Contract hash is not set.');
    }
    // ! TODO toPrefixedString() ?
    const key = `hash-${this.contractHash?.hash?.toHex()}`;

    if (!this.isOperatorArgs(params)) {
      const {
        params: { wasm, sender, paymentAmount, signingKeys, chainName },
        args: { tokenOwner, operator, keyName },
        waitForTransactionProcessed,
      } = params;

      if (keyName) {
        const wasmBytes = wasm || isApprovedForAllWASM;
        if (!wasmBytes) {
          throw new Error('Wasm file is missing.');
        }

        const runtimeArgs = RuntimeArgs.fromMap({
          nft_contract_hash: CLValue.newCLKey(Key.newKey(key)),
          token_owner: CLValue.newCLKey(this.getPrefixedString(tokenOwner)),
          operator: CLValue.newCLKey(this.getPrefixedString(operator)),
          key_name: CLValue.newCLString(keyName),
        });

        return this.callSession(
          wasmBytes,
          runtimeArgs,
          paymentAmount,
          sender,
          signingKeys,
          chainName,
          waitForTransactionProcessed
        );
      }
    }
    const { tokenOwner, operator } = params as OperatorArgs;
    const keyOwner = this.getPrefixedString(tokenOwner).bytes();
    const keySpender = this.getPrefixedString(operator).bytes();

    const finalBytes = new Uint8Array(keyOwner.length + keySpender.length);
    finalBytes.set(keyOwner);
    finalBytes.set(keySpender, keyOwner.length);

    const blaked = blake2b(finalBytes, { dkLen: 32 });
    const dictionaryItemKey = bytesToHex(blaked);
    const contractNamedKey: ParamDictionaryIdentifierContractNamedKey =
      new ParamDictionaryIdentifierContractNamedKey(
        key,
        'operators',
        dictionaryItemKey!
      );

    const identifier = new ParamDictionaryIdentifier(
      undefined,
      contractNamedKey,
      undefined,
      undefined
    );

    try {
      return (
        (
          await this.rpcClient.getDictionaryItemByIdentifier(null, identifier)
        ).storedValue.clValue?.toString() === 'true'
      );
    } catch (error) {
      if (error instanceof Error && error.toString().includes('Query failed')) {
        console.warn(`No approval found for ${keyOwner} and ${keySpender}`);
        return false;
      } else throw error;
    }
  }

  public async isAclWhitelisted(
    params: isAclWhitelistedParams
  ): Promise<boolean | TransactionResult> {
    if (!this.contractHash) {
      throw new Error('Contract hash is not set.');
    }
    // ! TODO toPrefixedString() ?
    const key = `hash-${this.contractHash?.hash?.toHex()}`;

    const entity = this.getPrefixedString(params);

    const dictionaryItemKey = this.getPrefixedString(params)
      .toPrefixedString()
      .replace(prefixRegex, '');
    const contractNamedKey: ParamDictionaryIdentifierContractNamedKey =
      new ParamDictionaryIdentifierContractNamedKey(
        key,
        'acl_whitelist',
        dictionaryItemKey!
      );

    const identifier = new ParamDictionaryIdentifier(
      undefined,
      contractNamedKey,
      undefined,
      undefined
    );

    try {
      return (
        (
          await this.rpcClient.getDictionaryItemByIdentifier(null, identifier)
        ).storedValue.clValue?.toString() === 'true'
      );
    } catch (error) {
      if (error instanceof Error && error.toString().includes('Query failed')) {
        console.warn(`No whiteListing for ${entity}`);
        return false;
      } else throw error;
    }
  }

  public setVariables(params: SetVariablesParams) {
    const {
      params: { sender, paymentAmount, signingKeys, chainName },
      args: {
        allowMinting,
        aclWhitelist,
        aclPackageMode,
        packageOperatorMode,
        operatorBurnMode,
      },
      waitForTransactionProcessed,
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({});

    if (allowMinting !== undefined) {
      runtimeArgs.insert('allow_minting', CLValue.newCLValueBool(allowMinting));
    }

    if (aclPackageMode !== undefined) {
      runtimeArgs.insert(
        'acl_package_mode',
        CLValue.newCLValueBool(aclPackageMode)
      );
    }

    if (operatorBurnMode !== undefined) {
      runtimeArgs.insert(
        'operator_burn_mode',
        CLValue.newCLValueBool(operatorBurnMode)
      );
    }

    if (aclWhitelist !== undefined) {
      const list = CLValue.newCLList(
        CLTypeKey,
        aclWhitelist.map((key) => CLValue.newCLKey(this.getPrefixedString(key)))
      );
      runtimeArgs.insert('acl_whitelist', list);
    }

    if (packageOperatorMode !== undefined) {
      runtimeArgs.insert(
        'package_operator_mode',
        CLValue.newCLValueBool(packageOperatorMode)
      );
    }

    return this.callEntrypoint(
      'set_variables',
      runtimeArgs,
      paymentAmount,
      sender,
      signingKeys,
      chainName,
      waitForTransactionProcessed
    );
  }

  public async metadata(tokenIdentifier: string) {
    if (!this.contractHash) {
      throw Error('Contract hash is not set.');
    }

    // ! TODO toPrefixedString() ?
    const key = `hash-${this.contractHash?.hash?.toHex()}`;

    const metadataToCheck: NFT_METADATA_KIND =
      NFT_METADATA_KIND[await this.metadataKind()];

    const mapMetadata = {
      [NFT_METADATA_KIND.CEP78]: 'metadata_cep78',
      [NFT_METADATA_KIND.NFT721]: 'metadata_nft721',
      [NFT_METADATA_KIND.Raw]: 'metadata_raw',
      [NFT_METADATA_KIND.CustomValidated]: 'metadata_custom_validated',
    };

    const dictionaryItemKey = tokenIdentifier;

    const contractNamedKey: ParamDictionaryIdentifierContractNamedKey =
      new ParamDictionaryIdentifierContractNamedKey(
        key,
        mapMetadata[metadataToCheck],
        dictionaryItemKey!
      );

    const identifier = new ParamDictionaryIdentifier(
      undefined,
      contractNamedKey,
      undefined,
      undefined
    );

    try {
      const metadata = (
        await this.rpcClient.getDictionaryItemByIdentifier(null, identifier)
      ).storedValue.clValue?.toJSON();

      return metadata;
    } catch (error) {
      if (error instanceof Error && error.toString().includes('Query failed')) {
        console.warn(`No metadata found for ${tokenIdentifier}`);
        return {};
      } else throw error;
    }
  }

  // Deprecated for 1.1 version
  public updatedReceipts(
    params: updatedReceiptsParams,
    callSessionWasm = true
  ) {
    const {
      params: { wasm, sender, paymentAmount, signingKeys, chainName },
      waitForTransactionProcessed,
    } = params;

    if (!this.contractHash) {
      throw Error('Contract package hash is not set.');
    }

    // ! TODO toPrefixedString() ?
    const key = `hash-${this.contractPackageHash?.hash?.toHex()}`;

    if (callSessionWasm) {
      const wasmBytes = wasm || UpdatedReceiptsWASM;
      if (!wasmBytes) {
        throw new Error('Wasm file is missing.');
      }

      const runtimeArgs = RuntimeArgs.fromMap({
        nft_contract_hash: CLValue.newCLKey(Key.newKey(key)),
      });

      return this.callSession(
        wasmBytes,
        runtimeArgs,
        paymentAmount,
        sender,
        signingKeys,
        chainName,
        waitForTransactionProcessed
      );
    }

    return this.callEntrypoint(
      'updated_receipts',
      RuntimeArgs.fromMap({}),
      paymentAmount,
      sender,
      signingKeys,
      chainName,
      waitForTransactionProcessed
    );
  }

  public async collectionName() {
    return this.queryContractData(['collection_name']);
  }

  public async collectionSymbol() {
    return this.queryContractData(['collection_symbol']);
  }

  public async tokenTotalSupply() {
    return this.queryContractData(['total_token_supply']);
  }

  public async numOfMintedTokens() {
    return this.queryContractData(['number_of_minted_tokens']);
  }

  public async allowMinting(): Promise<boolean> {
    const result = await this.queryContractData(['allow_minting']);
    return result === 'true';
  }

  public async mintingMode() {
    const internalValue = (await this.queryContractData([
      'minting_mode',
    ])) as unknown as number;
    return MINTING_MODE[internalValue] as keyof typeof MINTING_MODE;
  }

  public async whitelistMode() {
    const internalValue = (await this.queryContractData([
      'whitelist_mode',
    ])) as unknown as number;
    return WHITELIST_MODE[internalValue] as keyof typeof WHITELIST_MODE;
  }

  public async reportingMode() {
    const internalValue = (await this.queryContractData([
      'reporting_mode',
    ])) as unknown as number;
    return OWNER_REVERSE_LOOKUP_MODE[
      internalValue
    ] as keyof typeof OWNER_REVERSE_LOOKUP_MODE;
  }

  public async burnMode() {
    const internalValue = (await this.queryContractData([
      'burn_mode',
    ])) as unknown as number;
    return BURN_MODE[internalValue] as keyof typeof BURN_MODE;
  }

  public async holderMode() {
    const internalValue = (await this.queryContractData([
      'holder_mode',
    ])) as unknown as number;
    return NFT_HOLDER_MODE[internalValue] as keyof typeof NFT_HOLDER_MODE;
  }

  public async identifierMode() {
    const internalValue = (await this.queryContractData([
      'identifier_mode',
    ])) as unknown as number;
    return NFT_IDENTIFIER_MODE[
      internalValue
    ] as keyof typeof NFT_IDENTIFIER_MODE;
  }

  public async metadataMutability() {
    const internalValue = (await this.queryContractData([
      'metadata_mutability',
    ])) as unknown as number;
    return METADATA_MUTABILITY[
      internalValue
    ] as keyof typeof METADATA_MUTABILITY;
  }

  public async nftKind() {
    const internalValue = (await this.queryContractData([
      'nft_kind',
    ])) as unknown as number;
    return NFT_KIND[internalValue] as keyof typeof NFT_KIND;
  }

  public async metadataKind() {
    const internalValue = (await this.queryContractData([
      'nft_metadata_kind',
    ])) as unknown as number;
    return NFT_METADATA_KIND[internalValue] as keyof typeof NFT_METADATA_KIND;
  }

  public async ownershipMode() {
    const internalValue = (await this.queryContractData([
      'ownership_mode',
    ])) as unknown as number;
    return NFT_OWNERSHIP_MODE[internalValue] as keyof typeof NFT_OWNERSHIP_MODE;
  }

  public async jsonSchema() {
    const internalValue = (await this.queryContractData([
      'json_schema',
    ])) as unknown as number;
    return internalValue.toString();
  }

  /**
   * Returns the event mode of the CEP-78 token.
   *
   * @returns A `Promise` that resolves to a key of the `EVENTS_MODE` enum, indicating the event mode of the token.
   *
   * @remarks This method queries the `events_mode` field from the contract and returns the corresponding key from the `EVENTS_MODE` enum.
   */
  public async eventsMode(): Promise<keyof typeof EVENTS_MODE> {
    const internalValue = (await this.queryContractData([
      'events_mode',
    ])) as string;

    return EVENTS_MODE[internalValue] as keyof typeof EVENTS_MODE;
  }

  private addTokenIdentifierRuntimeArgs(
    runtimeArgs: RuntimeArgs,
    tokenId?: string,
    tokenHash?: string
  ) {
    if (tokenId) {
      runtimeArgs.insert('token_id', CLValue.newCLUint64(tokenId));
    } else if (tokenHash) {
      runtimeArgs.insert('token_hash', CLValue.newCLString(tokenHash));
    }
    return runtimeArgs;
  }

  private isOperatorArgs(obj: unknown): obj is OperatorArgs {
    return (
      typeof obj === 'object' &&
      obj !== null &&
      'tokenOwner' in obj &&
      'operator' in obj
    );
  }

  private isStoreBalanceOfParams(obj: unknown): obj is StoreBalanceOfParams {
    // We need to check if 'params' is an object and has the 'args' inside.
    return (
      typeof obj === 'object' &&
      obj !== null &&
      'args' in obj && // Ensure that 'args' is present
      typeof (obj as any).args === 'object' &&
      'tokenOwner' in (obj as any).args // Ensure 'tokenOwner' is present inside 'args'
    );
  }

  private getPrefixedString(entity: Entity): Key {
    if (entity instanceof PublicKey) {
      return Key.newKey(entity.accountHash().toPrefixedString());
    }
    return Key.newKey(entity.toPrefixedString());
  }
}
