import { blake2b } from '@noble/hashes/blake2b';
import { bytesToHex } from '@noble/hashes/utils';
import {
  AddressableEntityHash,
  Args as RuntimeArgs,
  CLTypeKey,
  CLValue,
  ContractHash,
  ContractPackageHash,
  Key,
  ParamDictionaryIdentifier,
  ParamDictionaryIdentifierContractNamedKey,
  PublicKey,
  SessionBuilder,
} from 'casper-js-sdk';
import Client from './client';
import {
  type SetApprovallForAllParams,
  type ApproveParams,
  type GetApprovedParams,
  type IsApprovedForAlldParams,
  type OwnerOfParams,
  type TransferParams,
  type BalanceOfParams,
  type StoreOwnerOfParams,
  type StoreBalanceOfParams,
  type MintParams,
  type BurnParams,
  type TokenMetadataParams,
  type RegisterParams,
  type InstallParams,
  type UpgradeParams,
  type SetVariablesParams,
  type updatedReceiptsParams,
  type TransactionResult,
  type OperatorArgs,
  type Entity,
  type isAclWhitelistedParams,
  BURN_MODE,
  EVENTS_MODE,
  METADATA_MUTABILITY,
  NAMED_KEY_CONVENTION_MODE,
  HOLDER_MODE,
  IDENTIFIER_MODE,
  NFT_KIND,
  NFT_METADATA_KIND,
  OWNERSHIP_MODE,
  OWNER_REVERSE_LOOKUP_MODE,
  WHITELIST_MODE,
  MINTING_MODE,
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

  /**
   * Installs the CEP-78 NFT contract on the Casper network.
   *
   * @param params - The installation parameters, including:
   *   - `wasm`: The compiled contract in `Uint8Array` format.
   *   - `paymentAmount`: The amount of payment required for contract installation.
   *   - `sender`: The public key of the account deploying the contract.
   *   - `chainName`: (Optional) The name of the network where the contract will be deployed.
   *   - `signingKeys`: (Optional) An array of private keys used for signing the transaction.
   *   - `args`: Contract-specific arguments, including:
   *     - `collectionName`: The name of the NFT collection.
   *     - `collectionSymbol`: The symbol representing the NFT collection.
   *     - `totalTokenSupply`: The total supply of NFTs in the collection.
   *     - `eventsMode`: (Optional) The mode in which events are emitted.
   *     - `ownershipMode`: The ownership model of the NFTs (Minter, Assigned, Transferable).
   *     - `nftKind`: (Optional) The kind of NFTs (Physical, Digital, Virtual).
   *     - `jsonSchema`: (Optional) The JSON schema defining metadata structure.
   *     - `nftMetadataKind`: The metadata standard used (CEP78, NFT721, Raw, CustomValidated).
   *     - `identifierMode`: The identifier mode for tokens (Ordinal, Hash).
   *     - `metadataMutability`: Specifies whether metadata is mutable or immutable.
   *     - `allowMinting`: (Optional) A boolean indicating whether minting is allowed.
   *     - `mintingMode`: (Optional) Specifies who can mint NFTs (Installer, Public, ACL).
   *     - `holderMode`: (Optional) Determines who can hold NFTs (Accounts, Contracts, Mixed).
   *     - `burnMode`: (Optional) Determines if NFTs can be burned (Burnable, NonBurnable).
   *     - `operatorBurnMode`: (Optional) Whether operators can burn NFTs.
   *     - `ownerReverseLookupMode`: (Optional) Mode for reverse lookup of ownership.
   *     - `packageOperatorMode`: (Optional) Whether package operators are enabled.
   *     - `aclWhitelist`: (Optional) A list of accounts/contracts allowed to interact.
   *     - `aclPackageMode`: (Optional) Whether ACL applies at the package level.
   *     - `whitelistMode`: (Optional) Whether the contract uses a whitelist (Unlocked, Locked).
   *     - `namedKeyConventionMode`: (Optional) The naming convention for contract keys.
   *     - `accessKeyName`: (Optional) The name for the access key (if using custom convention).
   *     - `hashKeyName`: (Optional) The name for the hash key (if using custom convention).
   *     - `transferFilterContract`: (Optional) A contract hash for filtering transfers.
   *   - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed.
   *
   * @returns A `Promise` resolving to `TransactionResult`, containing the transaction details.
   *
   * @throws Will throw an error if the Wasm file is missing, required arguments are not provided,
   * or if an error occurs during installation.
   *
   * @remarks
   * This method installs a new CEP-78 NFT contract on the Casper network. It requires a compiled Wasm contract file
   * and includes necessary arguments such as the collection name, symbol, supply, ownership mode, metadata type,
   * and other configurations.
   * If `waitForTransactionProcessed` is `true`, it waits for the transaction to be processed and returns the execution result.
   * Ensure that the Wasm file is valid and the required arguments are properly provided before invoking the method.
   */
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
        whitelistMode,
        aclWhitelist,
        aclPackageMode,
        namedKeyConventionMode,
        accessKeyName,
        hashKeyName,
        transferFilterContract,
      },
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      collection_name: CLValue.newCLString(collectionName),
      collection_symbol: CLValue.newCLString(collectionSymbol),
      total_token_supply: CLValue.newCLUint64(totalTokenSupply),
      identifier_mode: CLValue.newCLUint8(identifierMode),
      ownership_mode: CLValue.newCLUint8(ownershipMode),
      nft_metadata_kind: CLValue.newCLUint8(nftMetadataKind),
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
        aclWhitelist.map((key) => CLValue.newCLKey(this.getPrefixedString(key)))
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

    if (transferFilterContract !== undefined) {
      runtimeArgs.insert(
        'transfer_filter_contract',
        CLValue.newCLKey(this.getPrefixedString(transferFilterContract))
      );
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
            transactionInfo.transactionHash.toHex()
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
      throw new Error(
        `Error during installation runtime.\n${error}\n${(error as any)?.sourceErr?.data}`
      );
    }
  }

  /**
   * Upgrades an existing NFT contract on the Casper network.
   *
   * @param params - The upgrade parameters, including:
   *   - `wasm`: The compiled contract in `Uint8Array` format for the new version.
   *   - `paymentAmount`: The amount of payment required for the contract upgrade.
   *   - `sender`: The public key of the account initiating the upgrade.
   *   - `chainName`: (Optional) The name of the network where the contract is deployed.
   *   - `signingKeys`: (Optional) An array of private keys used for signing the transaction.
   *   - `args`: Contract-specific arguments, including:
   *     - `collectionName`: The name of the NFT collection.
   *     - `totalTokenSupply`: (Optional) The total supply of tokens for the collection.
   *     - `eventsMode`: (Optional) The mode in which events are emitted.
   *     - `aclPackageMode`: (Optional) Enables or disables Access Control List package mode.
   *     - `packageOperatorMode`: (Optional) Enables or disables package operator mode.
   *     - `operatorBurnMode`: (Optional) Enables or disables operator burn mode.
   *   - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed.
   *
   * @returns A `Promise` resolving to `TransactionResult`, containing the transaction details.
   *
   * @throws Will throw an error if the Wasm file is missing or if an error occurs during the upgrade process.
   *
   * @remarks
   * This method upgrades an existing NFT contract by deploying a new Wasm file while retaining existing data.
   * The `wasm` argument must be the compiled contract in `Uint8Array` format. If `eventsMode`, `aclPackageMode`,
   * `packageOperatorMode`, or `operatorBurnMode` are provided, they update the contract's behavior accordingly.
   * If `waitForTransactionProcessed` is `true`, the method waits for the transaction to be processed and returns the execution result.
   */
  public async upgrade(params: UpgradeParams): Promise<TransactionResult> {
    const {
      params: { wasm, paymentAmount, sender, chainName, signingKeys },
      args: {
        collectionName,
        totalTokenSupply,
        eventsMode,
        aclPackageMode,
        packageOperatorMode,
        operatorBurnMode,
      },
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      collection_name: CLValue.newCLString(collectionName),
    });

    if (totalTokenSupply !== undefined) {
      runtimeArgs.insert(
        'total_token_supply',
        CLValue.newCLUint64(totalTokenSupply)
      );
    }

    if (eventsMode !== undefined) {
      runtimeArgs.insert('events_mode', CLValue.newCLUint8(eventsMode));
    }

    if (aclPackageMode !== undefined) {
      runtimeArgs.insert(
        'acl_package_mode',
        CLValue.newCLValueBool(aclPackageMode)
      );
    }

    if (packageOperatorMode !== undefined) {
      runtimeArgs.insert(
        'package_operator_mode',
        CLValue.newCLValueBool(packageOperatorMode)
      );
    }

    if (operatorBurnMode !== undefined) {
      runtimeArgs.insert(
        'operator_burn_mode',
        CLValue.newCLValueBool(operatorBurnMode)
      );
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
            transactionInfo.transactionHash.toHex()
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
      throw new Error(
        `Error during upgrade runtime.\n${error}\n${(error as any)?.sourceErr?.data}`
      );
    }
  }

  /**
   * Mints a new NFT token and assigns it to the specified owner.
   *
   * @param params - The minting parameters, including:
   *   - `wasm`: (Optional) The compiled contract in `Uint8Array` format for minting via a session call.
   *   - `paymentAmount`: The payment amount required for executing the minting transaction.
   *   - `sender`: The public key of the account initiating the mint operation.
   *   - `chainName`: (Optional) The name of the network where the transaction will be executed.
   *   - `signingKeys`: (Optional) An array of private keys used for signing the transaction.
   *   - `args`: Minting-specific arguments, including:
   *     - `tokenOwner`: The public key or account address of the owner who will receive the newly minted token.
   *     - `tokenMetaData`: Metadata associated with the token (e.g., attributes, properties) as a JSON object.
   *     - `tokenHash`: (Optional) A unique hash identifier for the token.
   *   - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   * @param callSessionWasm - (Optional) If `true`, uses a session contract for minting instead of calling the contract entry point.
   *
   * @returns A `Promise` that resolves to a `TransactionResult` containing the transaction details.
   *
   * @throws Will throw an error if the contract hash is not set, the Wasm file is missing (when `callSessionWasm` is `true`),
   *         or if the transaction execution fails.
   *
   * @remarks
   * This method allows minting a new NFT token and assigning it to a specified owner. If `callSessionWasm` is set to `true`,
   * the minting process will use a session contract, requiring a Wasm file. Otherwise, the mint function will call the contract’s
   * `mint` entry point directly. If `tokenHash` is provided, it will be included in the runtime arguments.
   */
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

  /**
   * Burns (destroys) an existing NFT token, removing it from circulation.
   *
   * @param params - The parameters for burning a token, including:
   *   - `paymentAmount`: The payment amount required for executing the burn transaction.
   *   - `sender`: The public key of the account initiating the burn operation.
   *   - `chainName`: (Optional) The name of the network where the transaction will be executed.
   *   - `signingKeys`: (Optional) An array of private keys used for signing the transaction.
   *   - `args`: Burn-specific arguments, including:
   *     - `tokenId`: (Optional) The unique identifier of the token to be burned.
   *     - `tokenHash`: (Optional) The hash identifier of the token to be burned (used if `tokenId` is not provided).
   *   - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *
   * @returns A `Promise` that resolves to a `TransactionResult` containing the transaction details.
   *
   * @throws Will throw an error if both `tokenId` and `tokenHash` are missing or if the transaction execution fails.
   *
   * @remarks
   * This method allows an account to permanently remove an NFT token from circulation. The token can be identified
   * either by its numeric `tokenId` or its unique `tokenHash`. If both values are provided, only `tokenId` is used.
   * Once burned, the token cannot be recovered.
   */
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

  /**
   * Transfers an NFT token from one account to another.
   *
   * @param params - The parameters for the transfer operation, including:
   *   - `paymentAmount`: The amount of payment required for executing the transfer transaction.
   *   - `sender`: The public key of the sender initiating the transfer.
   *   - `chainName`: (Optional) The name of the network where the transaction will be executed.
   *   - `signingKeys`: (Optional) An array of private keys used for signing the transaction.
   *   - `args`: Transfer-specific arguments, including:
   *     - `target`: The public key or account address of the recipient.
   *     - `source`: The public key or account address of the current owner of the token.
   *     - `tokenId`: (Optional) The unique identifier of the token to be transferred.
   *     - `tokenHash`: (Optional) The hash identifier of the token to be transferred (used if `tokenId` is not provided).
   *   - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *   - `callSessionWasm`: (Optional) If `true`, executes the transfer using a session contract instead of an entrypoint call.
   *
   * @returns A `Promise` that resolves to a `TransactionResult` containing the transaction details.
   *
   * @throws Will throw an error if both `tokenId` and `tokenHash` are missing, if the WASM file is missing when using session mode, or if the transaction execution fails.
   *
   * @remarks
   * This method facilitates the transfer of NFT tokens from one account to another. The token can be identified
   * either by its numeric `tokenId` or its unique `tokenHash`. If both values are provided, only `tokenId` is used.
   * The function supports both direct entrypoint calls and session-based execution, allowing flexibility in contract interaction.
   */
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

  /**
   * Registers a new token owner in the NFT contract.
   *
   * @param params - The parameters for the registration operation, including:
   *   - `paymentAmount`: The amount of payment required for executing the transaction.
   *   - `sender`: The public key of the sender initiating the registration.
   *   - `chainName`: (Optional) The name of the network where the transaction will be executed.
   *   - `signingKeys`: (Optional) An array of private keys used for signing the transaction.
   *   - `args`: Registration-specific arguments, including:
   *     - `tokenOwner`: The public key or account address of the user to be registered as an NFT owner.
   *   - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *
   * @returns A `Promise` that resolves to a `TransactionResult` containing the transaction details.
   *
   * @throws Will throw an error if the transaction execution fails or if any required parameters are missing.
   *
   * @remarks
   * This method is used to register an account as an NFT owner within the contract. Registering an owner may be a prerequisite
   * for minting or receiving NFTs, depending on the contract's rules for OwnerReverseLookupMode (Complete/TransfersOnly).
   * The transaction includes the specified `tokenOwner` as a runtime argument, ensuring that the contract acknowledges the new owner.
   */
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

  /**
   * Grants approval to an operator to manage a specific NFT on behalf of the owner.
   *
   * @param params - The parameters for the approval operation, including:
   *   - `paymentAmount`: The amount of payment required for executing the transaction.
   *   - `sender`: The public key of the sender (token owner) granting approval.
   *   - `chainName`: (Optional) The name of the network where the transaction will be executed.
   *   - `signingKeys`: (Optional) An array of private keys used for signing the transaction.
   *   - `args`: Approval-specific arguments, including:
   *     - `operator`: The public key or account address of the operator receiving approval.
   *     - `tokenId`: (Optional) The ID of the token being approved.
   *     - `tokenHash`: (Optional) The hash of the token being approved (used if `tokenId` is not provided).
   *   - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *
   * @returns A `Promise` that resolves to a `TransactionResult` containing the transaction details.
   *
   * @throws Will throw an error if the transaction execution fails or if any required parameters are missing.
   *
   * @remarks
   * This method allows the owner of an NFT to grant approval to another account (operator) to transfer or manage
   * the specified NFT on their behalf. Either `tokenId` or `tokenHash` must be provided to specify the NFT being approved.
   */
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

  /**
   * Revokes approval from an operator, removing their ability to manage a specific NFT on behalf of the owner.
   *
   * @param params - The parameters for the revoke operation, including:
   *   - `paymentAmount`: The amount of payment required for executing the transaction.
   *   - `sender`: The public key of the sender (token owner) revoking the approval.
   *   - `chainName`: (Optional) The name of the network where the transaction will be executed.
   *   - `signingKeys`: (Optional) An array of private keys used for signing the transaction.
   *   - `args`: Revoke-specific arguments, including:
   *     - `operator`: The public key or account address of the operator whose approval is being revoked.
   *     - `tokenId`: (Optional) The ID of the token for which approval is being revoked.
   *     - `tokenHash`: (Optional) The hash of the token for which approval is being revoked (used if `tokenId` is not provided).
   *   - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *
   * @returns A `Promise` that resolves to a `TransactionResult` containing the transaction details.
   *
   * @throws Will throw an error if the transaction execution fails or if any required parameters are missing.
   *
   * @remarks
   * This method allows the owner of an NFT to revoke a previously granted approval, ensuring that the specified operator
   * can no longer transfer or manage the NFT on behalf of the owner. Either `tokenId` or `tokenHash` must be provided
   * to specify the NFT for which approval is being revoked.
   */
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

  /**
   * Grants or revokes approval for an operator to manage all of the sender's NFTs.
   *
   * @param params - The parameters for setting approval, including:
   *   - `paymentAmount`: The amount of payment required for executing the transaction.
   *   - `sender`: The public key of the sender (token owner) granting or revoking approval.
   *   - `chainName`: (Optional) The name of the network where the transaction will be executed.
   *   - `signingKeys`: (Optional) An array of private keys used for signing the transaction.
   *   - `args`: Approval-specific arguments, including:
   *     - `operator`: The public key or account address of the operator receiving or losing approval.
   *     - `approveAll`: A boolean indicating whether to grant (`true`) or revoke (`false`) approval for all tokens.
   *   - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *
   * @returns A `Promise` that resolves to a `TransactionResult` containing the transaction details.
   *
   * @throws Will throw an error if the transaction execution fails or if any required parameters are missing.
   *
   * @remarks
   * This method allows an NFT owner to approve or revoke an operator's ability to manage all NFTs owned by the sender.
   * When `approveAll` is `true`, the operator is given permission to transfer and manage all NFTs on behalf of the sender.
   * When `approveAll` is `false`, any previously granted permissions are revoked.
   */
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

  /**
   * Sets or updates the metadata associated with a specific token.
   *
   * @param params - The parameters for setting the token metadata, including:
   *   - `paymentAmount`: The amount of payment required for executing the transaction.
   *   - `sender`: The public key of the sender (token owner) who is setting the metadata.
   *   - `chainName`: (Optional) The name of the network where the transaction will be executed.
   *   - `signingKeys`: (Optional) An array of private keys used for signing the transaction.
   *   - `args`: Metadata-specific arguments, including:
   *     - `tokenMetaData`: A JSON object containing the metadata to be associated with the token.
   *   - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *
   * @returns A `Promise` that resolves to a `TransactionResult` containing the transaction details.
   *
   * @throws Will throw an error if the transaction execution fails or if any required parameters are missing.
   *
   * @remarks
   * This method allows the owner of a token to set or update its metadata. The metadata is provided as a JSON object,
   * and it can be used to store additional information about the token such as its properties, attributes, or description.
   * The metadata will be stored on the blockchain and can be accessed later by other applications interacting with the token.
   */
  public setTokenMetadata(
    params: TokenMetadataParams
  ): Promise<TransactionResult> {
    const {
      params: { sender, paymentAmount, signingKeys, chainName },
      args: { tokenMetaData, tokenId, tokenHash },
      waitForTransactionProcessed,
    } = params;

    const runtimeArgs = RuntimeArgs.fromMap({
      token_meta_data: CLValue.newCLString(JSON.stringify(tokenMetaData)),
    });

    if (tokenId) {
      runtimeArgs.insert('token_id', CLValue.newCLUint64(tokenId));
    } else if (tokenHash) {
      runtimeArgs.insert('token_hash', CLValue.newCLString(tokenHash));
    }

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

  /**
   * Retrieves the owner of a specific token by its identifier (token ID or hash).
   * This method can be used to query the owner of a token either by interacting with a smart contract session or directly through a query.
   *
   * @param params - The parameters for retrieving the token owner, which can be either an object containing mint parameters
   *   or a direct token identifier string:
   *   - `params`:
   *     - `wasm`: (Optional) The WASM file to interact with, if calling a session.
   *     - `sender`: (Optional) The public key of the sender (if calling a session).
   *     - `paymentAmount`: (Optional) The amount to be paid for executing the transaction.
   *     - `signingKeys`: (Optional) An array of signing keys.
   *     - `chainName`: (Optional) The name of the blockchain network.
   *     - `args`: Contains the token details:
   *       - `tokenId`: (Optional) The token ID for which the owner is being queried.
   *       - `tokenHash`: (Optional) The token hash for the queried token.
   *       - `keyName`: (Optional) The key name for querying the token owner.
   *     - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *   - `tokenIdentifier`: (Optional) A string directly representing the token ID or token hash.
   *
   * @returns A `Promise` that resolves to:
   *   - A `TransactionResult` object containing the transaction details if a session-based call is made.
   *   - A string containing the owner's address if the owner is successfully retrieved.
   *   - `undefined` if the owner cannot be found.
   *
   * @throws Will throw an error if the smart contract session fails or if the RPC client encounters an issue during the query.
   *
   * @remarks
   * This method can operate in two modes:
   * 1. **Session Mode**: When provided with the `wasm` and `keyName`, it will execute a session transaction to query the token owner.
   * 2. **Query Mode**: When given a direct `tokenIdentifier` (ID or hash), it will query the blockchain state to find the owner from the `token_owners` dictionary.
   *
   * If no owner is found, it will return `undefined` and log a warning message.
   */
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

  /**
   * Retrieves the balance of a specific token owner.
   * This method can be used to either execute a session transaction to query the balance or directly query the blockchain state for the balance.
   *
   * @param params - The parameters for retrieving the token owner's balance, which can be either an object containing balance parameters
   *   or a direct token owner entity:
   *   - `params`:
   *     - `wasm`: (Optional) The WASM file to interact with, if calling a session.
   *     - `sender`: (Optional) The public key of the sender (if calling a session).
   *     - `paymentAmount`: (Optional) The amount to be paid for executing the transaction.
   *     - `signingKeys`: (Optional) An array of signing keys.
   *     - `chainName`: (Optional) The name of the blockchain network.
   *     - `args`: Contains the balance query details:
   *       - `tokenOwner`: The public key or account address of the token owner whose balance is being queried.
   *       - `keyName`: (Optional) The key name for querying the balance.
   *     - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *   - `tokenOwner`: (Optional) A direct token owner entity, used to query the balance.
   *
   * @returns A `Promise` that resolves to:
   *   - A `TransactionResult` object containing the transaction details if a session-based call is made.
   *   - A string representing the balance of the token owner if the balance is successfully retrieved.
   *
   * @throws Will throw an error if the smart contract session fails or if the RPC client encounters an issue during the query.
   *
   * @remarks
   * This method can operate in two modes:
   * 1. **Session Mode**: When provided with the `wasm` and `keyName`, it will execute a session transaction to query the token balance.
   * 2. **Query Mode**: When given a direct `tokenOwner`, it will query the blockchain state to find the balance from the `balances` dictionary.
   *
   * If no balance is found for the given owner, it will return `'0'` and log a warning message.
   */
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

  /**
   * Retrieves the approved address for a specific token or token hash.
   * This method can be used to either execute a session transaction to query the approval or directly query the blockchain state for the approval information.
   *
   * @param params - The parameters for retrieving the approval, which can be either an object containing approval parameters or a direct token identifier:
   *   - `params`:
   *     - `wasm`: (Optional) The WASM file to interact with, if calling a session.
   *     - `sender`: (Optional) The public key of the sender (if calling a session).
   *     - `paymentAmount`: (Optional) The amount to be paid for executing the transaction.
   *     - `signingKeys`: (Optional) An array of signing keys.
   *     - `chainName`: (Optional) The name of the blockchain network.
   *     - `args`: Contains the approval query details:
   *       - `tokenId`: (Optional) The unique identifier of the token to check approval for.
   *       - `tokenHash`: (Optional) The hash of the token to check approval for.
   *       - `keyName`: (Optional) The key name for querying the approval.
   *     - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *   - `tokenIdentifier`: (Optional) A string representing the token identifier, used to query the approval.
   *
   * @returns A `Promise` that resolves to:
   *   - A `TransactionResult` object containing the transaction details if a session-based call is made.
   *   - A string representing the approved address if successfully retrieved.
   *   - `undefined` if no approval is found for the given token identifier.
   *
   * @throws Will throw an error if the smart contract session fails or if the RPC client encounters an issue during the query.
   *
   * @remarks
   * This method can operate in two modes:
   * 1. **Session Mode**: When provided with the `wasm` and `keyName`, it will execute a session transaction to query the token's approval information.
   * 2. **Query Mode**: When given a direct `tokenIdentifier`, it will query the blockchain state to find the approved address from the `approved` dictionary.
   *
   * If no approval is found for the given token, it will return an empty string and log a warning message.
   */
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

  /**
   * Checks whether an operator is approved to manage all tokens of a specific owner.
   * This method can be used to either execute a session transaction to query the approval or directly query the blockchain state for the approval information.
   *
   * @param params - The parameters for checking if the operator is approved for all tokens of the owner:
   *   - `params`:
   *     - `wasm`: (Optional) The WASM file to interact with, if calling a session.
   *     - `sender`: (Optional) The public key of the sender (if calling a session).
   *     - `paymentAmount`: (Optional) The amount to be paid for executing the transaction.
   *     - `signingKeys`: (Optional) An array of signing keys.
   *     - `chainName`: (Optional) The name of the blockchain network.
   *     - `args`: Contains the approval query details:
   *       - `tokenOwner`: The owner of the tokens to check for operator approval.
   *       - `operator`: The operator who is being checked for approval.
   *       - `keyName`: (Optional) The key name for querying the approval.
   *     - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *   - `operatorArgs`:
   *     - `tokenOwner`: The owner's address to check approval for.
   *     - `operator`: The operator address to check if they have approval.
   *
   * @returns A `Promise` that resolves to:
   *   - `true` if the operator is approved to manage all tokens of the owner.
   *   - `false` if the operator is not approved.
   *   - A `TransactionResult` if a session-based call is made.
   *
   * @throws Will throw an error if the smart contract session fails or if the RPC client encounters an issue during the query.
   *
   * @remarks
   * This method can operate in two modes:
   * 1. **Session Mode**: When provided with the `wasm` and `keyName`, it will execute a session transaction to query the operator's approval for managing all tokens of the owner.
   * 2. **Query Mode**: When provided with `tokenOwner` and `operator`, it will query the blockchain state to check if the operator has approval to manage all tokens of the owner by querying the `operators` dictionary.
   *
   * If no approval is found, it will return `false` and log a warning message.
   */
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

  /**
   * Checks if an entity is whitelisted in the Access Control List (ACL) of the smart contract.
   * This method queries the blockchain state to determine whether a specific entity (e.g., address or account) is part of the ACL whitelist.
   *
   * @param params - The parameters for checking if an entity is whitelisted in the ACL:
   *   - `params`: The entity whose whitelisting status needs to be checked. Typically, this is the address or key of the entity.
   *
   * @returns A `Promise` that resolves to:
   *   - `true` if the entity is whitelisted in the ACL.
   *   - `false` if the entity is not whitelisted in the ACL.
   *   - A `TransactionResult` if a session-based call is made.
   *
   * @throws Will throw an error if there is an issue with querying the blockchain state or if the contract hash is not set.
   *
   * @remarks
   * This method queries the `acl_whitelist` dictionary in the smart contract's storage to check if the provided entity is whitelisted.
   * If no whitelisting entry is found for the entity, it will return `false` and log a warning message.
   */
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

  /**
   * Sets various variables or configuration settings within the smart contract.
   * This function allows updating key parameters related to minting permissions, ACL (Access Control List), and package operation modes.
   *
   * @param params - The parameters for setting the contract variables:
   *   - `params`: Contains the details of the contract update operation:
   *     - `allowMinting`: A boolean value to enable or disable minting functionality within the contract.
   *     - `aclWhitelist`: An array of keys representing entities to be added to the whitelist for access control.
   *     - `aclPackageMode`: A boolean indicating whether the ACL package mode is enabled.
   *     - `packageOperatorMode`: A boolean indicating whether the package operator mode is enabled.
   *     - `operatorBurnMode`: A boolean indicating whether the operator can burn tokens.
   *   - `paymentAmount`: The amount of payment required for executing the transaction.
   *   - `sender`: The public key of the sender (the user initiating the operation).
   *   - `signingKeys`: (Optional) An array of private keys used to sign the transaction.
   *   - `chainName`: (Optional) The name of the network where the transaction will be deployed.
   *   - `waitForTransactionProcessed`: (Optional) If `true`, waits for the transaction to be processed before resolving.
   *
   * @returns A `Promise` that resolves to a `TransactionResult` containing the details of the transaction.
   *
   * @throws Will throw an error if any of the parameters are invalid or if the transaction execution fails.
   *
   * @remarks
   * This method allows contract administrators or authorized parties to modify the operational settings of the smart contract.
   * It manages minting permissions, access control lists (ACL), and various operational modes related to the package and operator functionality.
   */
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

  /**
   * Retrieves the metadata associated with a specific token identifier from the smart contract.
   * Depending on the metadata kind, it fetches the appropriate metadata type from the contract's storage.
   *
   * @param tokenIdentifier - A string representing the unique identifier of the token whose metadata is being queried.
   *
   * @returns A `Promise` that resolves to the metadata object of the token, or an empty object if no metadata is found.
   *
   * @throws Will throw an error if the contract hash is not set or if fetching the metadata fails.
   *
   * @remarks
   * This function first checks the kind of metadata associated with the contract (e.g., CEP78, NFT721, Raw, Custom Validated).
   * It then uses the corresponding metadata type to retrieve the token's metadata from the contract's dictionary.
   * If no metadata is found for the specified token identifier, an empty object is returned.
   */
  public async metadata(tokenIdentifier: string) {
    if (!this.contractHash) {
      throw Error('Contract hash is not set.');
    }

    // ! TODO toPrefixedString() ?
    const key = `hash-${this.contractHash?.hash?.toHex()}`;

    const metadataToCheck: NFT_METADATA_KIND =
      NFT_METADATA_KIND[await this.nftMetadataKind()];

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

  /**
   * Returns the collection name of the token.
   *
   * @returns A `Promise` that resolves to the collection name of the token.
   *
   * @remarks This method queries the `collection_name` field from the contract.
   */
  public async collectionName() {
    return this.queryContractData(['collection_name']);
  }

  /**
   * Returns the collection symbol of the token.
   *
   * @returns A `Promise` that resolves to the collection symbol of the token.
   *
   * @remarks This method queries the `collection_symbol` field from the contract.
   */
  public async collectionSymbol() {
    return this.queryContractData(['collection_symbol']);
  }

  /**
   * Returns the total supply of the token.
   *
   * @returns A `Promise` that resolves to the total supply of the token.
   *
   * @remarks This method queries the `total_token_supply` field from the contract.
   */
  public async tokenTotalSupply() {
    return this.queryContractData(['total_token_supply']);
  }

  /**
   * Returns the number of minted tokens.
   *
   * @returns A `Promise` that resolves to the number of minted tokens.
   *
   * @remarks This method queries the `number_of_minted_tokens` field from the contract.
   */
  public async numOfMintedTokens() {
    return this.queryContractData(['number_of_minted_tokens']);
  }

  /**
   * Returns whether minting is allowed for the token.
   *
   * @returns A `Promise` that resolves to a boolean indicating whether minting is allowed.
   *
   * @remarks This method queries the `allow_minting` field from the contract and returns a boolean value.
   */
  public async allowMinting(): Promise<boolean> {
    const result = await this.queryContractData(['allow_minting']);
    return result === 'true';
  }

  /**
   * Returns the minting mode of the token.
   *
   * @returns A `Promise` that resolves to a key of the `MINTING_MODE` enum, indicating the minting mode of the token.
   *
   * @remarks This method queries the `minting_mode` field from the contract and returns the corresponding key from the `MINTING_MODE` enum.
   */
  public async mintingMode() {
    const internalValue = (await this.queryContractData([
      'minting_mode',
    ])) as unknown as number;
    return MINTING_MODE[internalValue] as keyof typeof MINTING_MODE;
  }

  /**
   * Returns the whitelist mode of the token.
   *
   * @returns A `Promise` that resolves to a key of the `WHITELIST_MODE` enum, indicating the whitelist mode of the token.
   *
   * @remarks This method queries the `whitelist_mode` field from the contract and returns the corresponding key from the `WHITELIST_MODE` enum.
   */
  public async whitelistMode() {
    const internalValue = (await this.queryContractData([
      'whitelist_mode',
    ])) as unknown as number;
    return WHITELIST_MODE[internalValue] as keyof typeof WHITELIST_MODE;
  }

  /**
   * Returns the reporting mode of the token.
   *
   * @returns A `Promise` that resolves to a key of the `OWNER_REVERSE_LOOKUP_MODE` enum, indicating the reporting mode of the token.
   *
   * @remarks This method queries the `reporting_mode` field from the contract and returns the corresponding key from the `OWNER_REVERSE_LOOKUP_MODE` enum.
   */
  public async reportingMode() {
    const internalValue = (await this.queryContractData([
      'reporting_mode',
    ])) as unknown as number;
    return OWNER_REVERSE_LOOKUP_MODE[
      internalValue
    ] as keyof typeof OWNER_REVERSE_LOOKUP_MODE;
  }

  /**
   * Returns the burn mode of the token.
   *
   * @returns A `Promise` that resolves to a key of the `BURN_MODE` enum, indicating the burn mode of the token.
   *
   * @remarks This method queries the `burn_mode` field from the contract and returns the corresponding key from the `BURN_MODE` enum.
   */
  public async burnMode() {
    const internalValue = (await this.queryContractData([
      'burn_mode',
    ])) as unknown as number;
    return BURN_MODE[internalValue] as keyof typeof BURN_MODE;
  }

  /**
   * Retrieves the `operatorBurnMode` status from the contract.
   * This mode indicates whether the operator can burn tokens in the contract.
   *
   * @returns A `Promise` that resolves to a boolean:
   *   - `true` if the `operatorBurnMode` is enabled, meaning the operator can burn tokens.
   *   - `false` if the `operatorBurnMode` is disabled, meaning the operator cannot burn tokens.
   *
   * @throws Will throw an error if there is an issue querying the contract data.
   */
  public async operatorBurnMode(): Promise<boolean> {
    const result = await this.queryContractData(['operator_burn_mode']);
    return result === 'true';
  }

  /**
   * Returns the holder mode of the token.
   *
   * @returns A `Promise` that resolves to a key of the `HOLDER_MODE` enum, indicating the holder mode of the token.
   *
   * @remarks This method queries the `holder_mode` field from the contract and returns the corresponding key from the `HOLDER_MODE` enum.
   */
  public async holderMode() {
    const internalValue = (await this.queryContractData([
      'holder_mode',
    ])) as unknown as number;
    return HOLDER_MODE[internalValue] as keyof typeof HOLDER_MODE;
  }

  /**
   * Returns the identifier mode of the token.
   *
   * @returns A `Promise` that resolves to a key of the `IDENTIFIER_MODE` enum, indicating the identifier mode of the token.
   *
   * @remarks This method queries the `identifier_mode` field from the contract and returns the corresponding key from the `IDENTIFIER_MODE` enum.
   */
  public async identifierMode() {
    const internalValue = (await this.queryContractData([
      'identifier_mode',
    ])) as unknown as number;
    return IDENTIFIER_MODE[internalValue] as keyof typeof IDENTIFIER_MODE;
  }

  /**
   * Returns the metadata mutability mode of the token.
   *
   * @returns A `Promise` that resolves to a key of the `METADATA_MUTABILITY` enum, indicating the metadata mutability mode of the token.
   *
   * @remarks This method queries the `metadata_mutability` field from the contract and returns the corresponding key from the `METADATA_MUTABILITY` enum.
   */
  public async metadataMutability() {
    const internalValue = (await this.queryContractData([
      'metadata_mutability',
    ])) as unknown as number;
    return METADATA_MUTABILITY[
      internalValue
    ] as keyof typeof METADATA_MUTABILITY;
  }

  /**
   * Returns the kind of the NFT.
   *
   * @returns A `Promise` that resolves to a key of the `NFT_KIND` enum, indicating the NFT kind.
   *
   * @remarks This method queries the `nft_kind` field from the contract and returns the corresponding key from the `NFT_KIND` enum.
   */
  public async nftKind() {
    const internalValue = (await this.queryContractData([
      'nft_kind',
    ])) as unknown as number;
    return NFT_KIND[internalValue] as keyof typeof NFT_KIND;
  }

  /**
   * Returns the metadata kind of the token.
   *
   * @returns A `Promise` that resolves to a key of the `NFT_METADATA_KIND` enum, indicating the metadata kind of the token.
   *
   * @remarks This method queries the `nft_metadata_kind` field from the contract and returns the corresponding key from the `NFT_METADATA_KIND` enum.
   */
  public async nftMetadataKind() {
    const internalValue = (await this.queryContractData([
      'nft_metadata_kind',
    ])) as unknown as number;
    return NFT_METADATA_KIND[internalValue] as keyof typeof NFT_METADATA_KIND;
  }

  /**
   * Returns the ownership mode of the token.
   *
   * @returns A `Promise` that resolves to a key of the `OWNERSHIP_MODE` enum, indicating the ownership mode of the token.
   *
   * @remarks This method queries the `ownership_mode` field from the contract and returns the corresponding key from the `OWNERSHIP_MODE` enum.
   */
  public async ownershipMode() {
    const internalValue = (await this.queryContractData([
      'ownership_mode',
    ])) as unknown as number;
    return OWNERSHIP_MODE[internalValue] as keyof typeof OWNERSHIP_MODE;
  }

  /**
   * Retrieves the `packageOperatorMode` status from the contract.
   * This mode indicates whether the package operator mode is enabled in the contract.
   *
   * @returns A `Promise` that resolves to a boolean:
   *   - `true` if the `packageOperatorMode` is enabled in the contract.
   *   - `false` if the `packageOperatorMode` is disabled in the contract.
   *
   * @throws Will throw an error if there is an issue querying the contract data.
   */
  public async packageOperatorMode(): Promise<boolean> {
    const result = await this.queryContractData(['package_operator_mode']);
    return result === 'true';
  }

  /**
   * Retrieves the `aclPackageMode` status from the contract.
   * This mode indicates whether the ACL (Access Control List) package is enabled or not in the contract.
   *
   * @returns A `Promise` that resolves to a boolean:
   *   - `true` if the `aclPackageMode` is enabled in the contract.
   *   - `false` if the `aclPackageMode` is disabled in the contract.
   *
   * @throws Will throw an error if there is an issue querying the contract data.
   */
  public async aclPackageMode(): Promise<boolean> {
    const result = await this.queryContractData(['acl_package_mode']);
    return result === 'true';
  }

  /**
   * Returns the JSON schema of the contract.
   *
   * @returns A `Promise` that resolves to the JSON schema of the contract.
   *
   * @remarks This method queries the `json_schema` field from the contract and returns the schema as a string.
   */
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

  // ! TODO toPrefixedString() ?
  // Error: prefix is not found, source: contract-0x, see Key.newKey()
  private getPrefixedString(entity: Entity): Key {
    if (entity instanceof PublicKey) {
      return Key.newKey(entity.accountHash().toPrefixedString());
    }
    if (
      entity instanceof ContractHash ||
      entity instanceof ContractPackageHash
    ) {
      return Key.newKey(`hash-${entity.hash.toHex()}`);
    }
    return Key.newKey((entity as AddressableEntityHash).toPrefixedString());
  }
}
