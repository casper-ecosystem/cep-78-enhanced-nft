import {
  Args,
  CLTypeKey,
  CLValue,
  ContractHash,
  ContractPackageHash,
  ExecutionResult,
  Key,
  KeyAlgorithm,
  PrivateKey,
  PutTransactionResult,
  RpcClient,
  StateGetDictionaryResult,
  TransactionProcessedPayload,
} from 'casper-js-sdk';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  EVENTS_MODE,
  InstallParams,
  CEP78Client,
  UpgradeParams,
  TransferParams,
  ApproveParams,
  MintParams,
  BurnParams,
  METADATA_MUTABILITY,
  NFT_IDENTIFIER_MODE,
  NFT_METADATA_KIND,
  NFT_OWNERSHIP_MODE,
  BURN_MODE,
  MINTING_MODE,
  NFT_HOLDER_MODE,
  NFT_KIND,
  OWNER_REVERSE_LOOKUP_MODE,
  WHITELIST_MODE,
} from '../../src';

describe('CEP78Client Unit', () => {
  describe('CEP78Client - setContractHash', () => {
    let client: CEP78Client;
    beforeEach(() => {
      // Initializing a new CEP78Client instance for each test
      client = new CEP78Client('http://mock-rpc-url');
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it('should correctly set the contract hash and contract package hash', () => {
      const contractHash = 'contract-hash-0x';
      const contractPackageHash = 'contract-package-0x';
      // Spy on the method to see if the contract hash is set properly
      vi.spyOn(ContractHash, 'newContract').mockImplementation(() => {
        return {} as ContractHash;
      });
      vi.spyOn(ContractPackageHash, 'newContractPackage').mockImplementation(
        () => {
          return {} as ContractPackageHash;
        }
      );
      const result = client.setContractHash(contractHash, contractPackageHash);
      // Check if the correct methods were called for both contract hash and contract package hash
      expect(ContractHash.newContract).toHaveBeenCalledWith('0x');
      expect(ContractPackageHash.newContractPackage).toHaveBeenCalledWith('0x');
      expect(result).toBeInstanceOf(CEP78Client);
    });

    it('should throw an error if contract hash is not provided', () => {
      // Providing invalid contract hash
      expect(() => client.setContractHash('')).toThrowError(
        'Contract hash must be provided.'
      );
    });

    it('should correctly remove prefixes from the contract hash and contract package hash', () => {
      const contractHashWithPrefix = 'hash-12345';
      const contractPackageHashWithPrefix = 'package-0x';
      // Mock implementation of `ContractHash` and `ContractPackageHash`
      vi.spyOn(ContractHash, 'newContract').mockImplementation(() => {
        return {} as ContractHash;
      });
      vi.spyOn(ContractPackageHash, 'newContractPackage').mockImplementation(
        () => {
          return {} as ContractPackageHash;
        }
      );
      client.setContractHash(
        contractHashWithPrefix,
        contractPackageHashWithPrefix
      );
      // Ensure the prefixes are correctly removed
      expect(ContractHash.newContract).toHaveBeenCalledWith('12345');
      expect(ContractPackageHash.newContractPackage).toHaveBeenCalledWith('0x');
    });

    it('should handle optional contract package hash', () => {
      const contractHash = 'contract-hash-0x';
      // Mock implementation of `ContractHash`
      vi.spyOn(ContractHash, 'newContract').mockImplementation(() => {
        return {} as ContractHash;
      });
      // No contract package hash provided, so it should still work
      const result = client.setContractHash(contractHash);
      expect(ContractHash.newContract).toHaveBeenCalledWith('0x');
      expect(result).toBeInstanceOf(CEP78Client);
    });
  });

  describe('CEP78Client - Event Stream', () => {
    let client: CEP78Client;
    let mockSseUrl: string;

    beforeEach(() => {
      client = new CEP78Client(
        'http://mock-rpc-url',
        'http://mock-sse-url',
        'testnet'
      );
      mockSseUrl = 'http://mock-sse-url';
    });

    it('should call startEventStream and return the updated CEP78Client instance', () => {
      // Spy on the super class method
      const startEventStreamSpy = vi
        .spyOn(CEP78Client.prototype, 'startEventStream')
        .mockReturnThis();

      const result = client.startEventStream(mockSseUrl);
      expect(startEventStreamSpy).toHaveBeenCalledWith(mockSseUrl);
      expect(result).toBe(client);
    });

    it('should call stopEventStream and return the updated CEP78Client instance', () => {
      // Spy on the super class method
      const stopEventStreamSpy = vi
        .spyOn(CEP78Client.prototype, 'stopEventStream')
        .mockReturnThis();

      // Call stopEventStream and check that it works
      const result = client.stopEventStream();
      expect(stopEventStreamSpy).toHaveBeenCalled();
      expect(result).toBe(client); // Expecting the same instance to be returned
    });

    it('should handle undefined sseUrl gracefully', () => {
      const clientWithUndefinedSseUrl = new CEP78Client(
        'http://mock-rpc-url',
        undefined, // undefined sseUrl for testing
        'testnet'
      );

      const startEventStreamSpy = vi
        .spyOn(CEP78Client.prototype, 'startEventStream')
        .mockReturnThis();

      const result = clientWithUndefinedSseUrl.startEventStream(
        'http://mock-sse-url'
      );
      expect(startEventStreamSpy).toHaveBeenCalledWith(mockSseUrl);
      expect(result).toBe(clientWithUndefinedSseUrl);
    });
  });

  describe('CEP78Client - install', () => {
    let client: CEP78Client;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockParams: InstallParams = {
      params: {
        wasm: new Uint8Array(),
        paymentAmount: '1000',
        sender: key.publicKey,
        chainName: 'testnet',
        signingKeys: [key],
      },
      args: {
        collectionName: 'CEP78',
        collectionSymbol: 'CEP78',
        totalTokenSupply: String(1000000),
        eventsMode: EVENTS_MODE.CES,
        ownershipMode: NFT_OWNERSHIP_MODE.Minter,
        nftMetadataKind: NFT_METADATA_KIND.CEP78,
        identifierMode: NFT_IDENTIFIER_MODE.Ordinal,
        metadataMutability: METADATA_MUTABILITY.Immutable,
      },
      waitForTransactionProcessed: false,
    };

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url');
      vi.spyOn(client['_rpcClient'], 'putTransaction').mockResolvedValue({
        transactionHash: 'mockTransactionHash',
      } as unknown as PutTransactionResult);
      vi.spyOn(client, 'waitForTransactionProcessed').mockResolvedValue({
        transactionProcessedPayload: {
          executionResult: { errorMessage: '' } as ExecutionResult,
        } as unknown as TransactionProcessedPayload,
      });
    });

    it('should successfully install a contract', async () => {
      const result = await client.install(mockParams);

      expect(client['_rpcClient'].putTransaction).toHaveBeenCalled();
      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
      });
    });

    it('should call waitForTransactionProcessed if waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };

      await client.install(paramsWithWait);

      expect(client.waitForTransactionProcessed).toHaveBeenCalledWith(
        'mockTransactionHash'
      );
    });

    it('should successfully install a contract if waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      const result = await client.install(paramsWithWait);

      expect(client['_rpcClient'].putTransaction).toHaveBeenCalled();

      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
        executionResult: { errorMessage: '' },
      });
    });

    it('should handle errors during transaction installation', async () => {
      const errorMessage = 'error during installation';
      vi.spyOn(client['_rpcClient'], 'putTransaction').mockRejectedValueOnce(
        new Error(errorMessage)
      );

      await expect(client.install(mockParams)).rejects.toThrow(
        `Error during installation runtime.\nError: ${errorMessage}`
      );
    });
  });

  describe('CEP78Client - upgrade', () => {
    let client: CEP78Client;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockParams: UpgradeParams = {
      params: {
        wasm: new Uint8Array(),
        paymentAmount: '1000',
        sender: key.publicKey,
        chainName: 'testnet',
        signingKeys: [key],
      },
      args: {
        collectionName: 'CEP78',
        eventsMode: EVENTS_MODE.CES,
      },
      waitForTransactionProcessed: false,
    };

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url');
      vi.spyOn(client['_rpcClient'], 'putTransaction').mockResolvedValue({
        transactionHash: 'mockTransactionHash',
      } as unknown as PutTransactionResult);
      vi.spyOn(client, 'waitForTransactionProcessed').mockResolvedValue({
        transactionProcessedPayload: {
          executionResult: { errorMessage: '' } as ExecutionResult,
        } as unknown as TransactionProcessedPayload,
      });
    });

    it('should successfully upgrade a contract', async () => {
      const result = await client.upgrade(mockParams);

      expect(client['_rpcClient'].putTransaction).toHaveBeenCalled();
      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
      });
    });

    it('should call waitForTransactionProcessed if waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };

      await client.upgrade(paramsWithWait);

      expect(client.waitForTransactionProcessed).toHaveBeenCalledWith(
        'mockTransactionHash'
      );
    });

    it('should successfully upgrade a contract if waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      const result = await client.upgrade(paramsWithWait);

      expect(client['_rpcClient'].putTransaction).toHaveBeenCalled();

      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
        executionResult: { errorMessage: '' },
      });
    });

    it('should handle errors during transaction upgradeation', async () => {
      const errorMessage = 'error during upgrade';
      vi.spyOn(client['_rpcClient'], 'putTransaction').mockRejectedValueOnce(
        new Error(errorMessage)
      );

      await expect(client.upgrade(mockParams)).rejects.toThrow(
        `Error during upgrade runtime.\nError: ${errorMessage}`
      );
    });
  });

  describe('CEP78Client - transfer', () => {
    let client: CEP78Client;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const key2 = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockParams: TransferParams = {
      params: {
        sender: key.publicKey,
        paymentAmount: '1000',
        signingKeys: [key],
        chainName: 'testnet',
      },
      args: {
        target: key.publicKey,
        source: key2.publicKey,
        tokenHash: 'tokenHash',
      },
      waitForTransactionProcessed: false,
    };

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url');
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
      });
      vi.spyOn(client, 'waitForTransactionProcessed').mockResolvedValue({
        transactionProcessedPayload: {
          executionResult: { errorMessage: '' } as ExecutionResult,
        } as unknown as TransactionProcessedPayload,
      });
    });

    it('should successfully transfer', async () => {
      const result = await client.transfer(mockParams);

      // Check that callEntrypoint was called with correct parameters
      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'transfer',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        mockParams.waitForTransactionProcessed
      );

      // Check the result
      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
      });
    });

    it('should call callEntrypoint with correct runtime arguments', async () => {
      await client.transfer(mockParams);

      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Assert the runtime arguments for transfer
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          target_key: CLValue.newCLKey(
            Key.newKey(key.publicKey.accountHash().toPrefixedString())
          ),
          source_key: CLValue.newCLKey(
            Key.newKey(key2.publicKey.accountHash().toPrefixedString())
          ),
          token_hash: CLValue.newCLString('tokenHash'),
        })
      );
    });

    it('should successfully transfer when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.transfer(paramsWithWait);

      // Check the result
      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });
    });

    it('should handle the case when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      await client.transfer(paramsWithWait);

      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'transfer',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        true
      );
    });

    it('should handle errors during the transfer process', async () => {
      const errorMessage = 'Error during transfer.';
      vi.spyOn(client as any, 'callEntrypoint').mockRejectedValueOnce(
        new Error(errorMessage)
      );

      await expect(client.transfer(mockParams)).rejects.toThrow(
        'Error during transfer.'
      );
    });
  });

  describe('CEP78Client - approve', () => {
    let client: CEP78Client;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const spenderKey = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockParams: ApproveParams = {
      params: {
        sender: key.publicKey,
        paymentAmount: '1000',
        signingKeys: [key],
        chainName: 'testnet',
      },
      args: {
        operator: spenderKey.publicKey,
        tokenHash: 'tokenHash',
      },
      waitForTransactionProcessed: false,
    };

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url');
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
      });
      vi.spyOn(client, 'waitForTransactionProcessed').mockResolvedValue({
        transactionProcessedPayload: {
          executionResult: { errorMessage: '' } as ExecutionResult,
        } as unknown as TransactionProcessedPayload,
      });
    });

    it('should successfully execute approve', async () => {
      const result = await client.approve(mockParams);

      // Verify callEntrypoint was called with correct parameters
      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'approve',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        mockParams.waitForTransactionProcessed
      );

      // Validate result
      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
      });
    });

    it('should call callEntrypoint with correct runtime arguments', async () => {
      await client.approve(mockParams);

      // Retrieve runtimeArgs from spy call
      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate runtime arguments for approve
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          operator: CLValue.newCLKey(
            Key.newKey(spenderKey.publicKey.accountHash().toPrefixedString())
          ),
          token_hash: CLValue.newCLString('tokenHash'),
        })
      );
    });

    it('should successfully execute approve when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.approve(paramsWithWait);

      // Validate result
      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });
    });

    it('should handle the case when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      await client.approve(paramsWithWait);

      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'approve',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        true
      );
    });

    it('should handle errors during the approve process', async () => {
      const errorMessage = 'Error during approve.';
      vi.spyOn(client as any, 'callEntrypoint').mockRejectedValueOnce(
        new Error(errorMessage)
      );

      await expect(client.approve(mockParams)).rejects.toThrow(
        'Error during approve.'
      );
    });
  });

  describe('CEP78Client - mint', () => {
    let client: CEP78Client;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const ownerKey = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockParams: MintParams = {
      params: {
        sender: key.publicKey,
        paymentAmount: '1000',
        signingKeys: [key],
        chainName: 'testnet',
      },
      args: {
        tokenOwner: ownerKey.publicKey,
        tokenMetaData: {
          ipfs_cid: 'ipfs_cid',
          color: 'Blue',
          ucid: 'ucid',
        },
        tokenHash: 'tokenHash',
      },
      waitForTransactionProcessed: false,
    };

    const contractHash =
      'hash-a84b9f15e57097579cb651bc3eec5143972c8c9ea153bb26d07367f9d41a767b';

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url').setContractHash(
        contractHash
      );
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
      });
      vi.spyOn(client, 'waitForTransactionProcessed').mockResolvedValue({
        transactionProcessedPayload: {
          executionResult: { errorMessage: '' } as ExecutionResult,
        } as unknown as TransactionProcessedPayload,
      });
    });

    it('should successfully execute mint', async () => {
      const result = await client.mint(mockParams);

      // Verify callEntrypoint was called with correct parameters
      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'mint',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        mockParams.waitForTransactionProcessed
      );

      // Validate result
      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
      });
    });

    it('should call callEntrypoint with correct runtime arguments', async () => {
      await client.mint(mockParams);

      // Retrieve runtimeArgs from spy call
      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate runtime arguments for mint
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          token_owner: CLValue.newCLKey(
            Key.newKey(ownerKey.publicKey.accountHash().toPrefixedString())
          ),
          token_meta_data: CLValue.newCLString(
            '{"ipfs_cid":"ipfs_cid","color":"Blue","ucid":"ucid"}'
          ),
          token_hash: CLValue.newCLString(mockParams.args.tokenHash!),
        })
      );
    });

    it('should successfully execute mint when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.mint(paramsWithWait);

      // Validate result
      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });
    });

    it('should handle the case when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      await client.mint(paramsWithWait);

      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'mint',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        true
      );
    });

    it('should handle errors during the mint process', async () => {
      const errorMessage = 'Error during mint.';
      vi.spyOn(client as any, 'callEntrypoint').mockRejectedValueOnce(
        new Error(errorMessage)
      );

      await expect(client.mint(mockParams)).rejects.toThrow(
        'Error during mint.'
      );
    });
  });

  describe('CEP78Client - burn', () => {
    let client: CEP78Client;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockParams: BurnParams = {
      params: {
        sender: key.publicKey,
        paymentAmount: '1000',
        signingKeys: [key],
        chainName: 'testnet',
      },
      args: {
        tokenHash: 'tokenHash',
      },
      waitForTransactionProcessed: false,
    };

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url');
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
      });
      vi.spyOn(client, 'waitForTransactionProcessed').mockResolvedValue({
        transactionProcessedPayload: {
          executionResult: { errorMessage: '' } as ExecutionResult,
        } as unknown as TransactionProcessedPayload,
      });
    });

    it('should successfully execute burn', async () => {
      const result = await client.burn(mockParams);

      // Verify callEntrypoint was called with correct parameters
      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'burn',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        mockParams.waitForTransactionProcessed
      );

      // Validate result
      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
      });
    });

    it('should call callEntrypoint with correct runtime arguments', async () => {
      await client.burn(mockParams);

      // Retrieve runtimeArgs from spy call
      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate runtime arguments for burn
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          token_hash: CLValue.newCLString(mockParams.args.tokenHash),
        })
      );
    });

    it('should successfully execute burn when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.burn(paramsWithWait);

      // Validate result
      expect(result).toEqual({
        transactionInfo: { transactionHash: 'mockTransactionHash' },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });
    });

    it('should handle the case when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      await client.burn(paramsWithWait);

      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'burn',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        true
      );
    });

    it('should handle errors during the burn process', async () => {
      const errorMessage = 'Error during burn.';
      vi.spyOn(client as any, 'callEntrypoint').mockRejectedValueOnce(
        new Error(errorMessage)
      );

      await expect(client.burn(mockParams)).rejects.toThrow(
        'Error during burn.'
      );
    });
  });

  describe('CEP78Client - balanceOf', () => {
    let client: CEP78Client;
    let mockRpcClient: RpcClient;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const contractHash =
      'hash-a84b9f15e57097579cb651bc3eec5143972c8c9ea153bb26d07367f9d41a767b';
    const mockBalance = '1000';

    beforeEach(() => {
      mockRpcClient = {
        getDictionaryItemByIdentifier: vi.fn(),
      } as unknown as RpcClient;

      client = new CEP78Client('http://mock-rpc-url');
      client['_rpcClient'] = mockRpcClient;
      client.setContractHash(contractHash);

      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValue({
        storedValue: {
          clValue: CLValue.newCLUInt512(mockBalance),
        },
      } as StateGetDictionaryResult);
    });

    it('should return the balance as a string when found', async () => {
      const balance = await client.balanceOf(key.publicKey);
      expect(balance).toBe(mockBalance);
    });

    it('should return "0" when balance is not found', async () => {
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValue({
        storedValue: { clValue: undefined },
      } as StateGetDictionaryResult);

      const balance = await client.balanceOf(key.publicKey);
      expect(balance).toBe('0');
    });

    it('should log a warning and return "0" when query fails with a missing balance', async () => {
      const consoleWarnSpy = vi
        .spyOn(console, 'warn')
        .mockImplementation(() => {});
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockRejectedValue(new Error('Error: Query failed'));

      const balance = await client.balanceOf(key.publicKey);
      expect(balance).toBe('0');
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        `No balance found for ${key.publicKey.accountHash().toPrefixedString()}`
      );
    });

    it('should throw an error if the query fails due to another issue', async () => {
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockRejectedValue(new Error('Unexpected RPC error'));

      await expect(client.balanceOf(key.publicKey)).rejects.toThrow(
        'Unexpected RPC error'
      );
    });

    it('should construct the correct dictionary key', async () => {
      await client.balanceOf(key.publicKey);

      const dictionaryIdentifier = (mockRpcClient as any)
        .getDictionaryItemByIdentifier.mock.calls[0][1];

      expect(dictionaryIdentifier.contractNamedKey).toEqual({
        key: contractHash,
        dictionaryName: 'balances',
        dictionaryItemKey: expect.any(String),
      });
    });

    it('should throw an error when contract hash is not set in balanceOf', async () => {
      const clientWithoutContractHash = new CEP78Client('http://mock-rpc-url');
      await expect(
        clientWithoutContractHash.balanceOf(key.publicKey)
      ).rejects.toThrow('Contract hash is not set.');
    });
  });

  describe('CEP78Client - Getter Methods', () => {
    let client: CEP78Client;

    const mockCollectionName = 'MyCollection';
    const mockCollectionSymbol = 'MYC';
    const mockTotalTokenSupply = '1000000';
    const mockNumOfMintedTokens = '500000';
    const mockAllowMinting = 'true';
    const mockMintingMode = MINTING_MODE.Public;
    const mockWhitelistMode = WHITELIST_MODE.Unlocked;
    const mockReportingMode = OWNER_REVERSE_LOOKUP_MODE.Complete;
    const mockBurnMode = BURN_MODE.Burnable;
    const mockHolderMode = NFT_HOLDER_MODE.Mixed;
    const mockIdentifierMode = NFT_IDENTIFIER_MODE.Hash;
    const mockMetadataMutability = METADATA_MUTABILITY.Immutable;
    const mockNftKind = NFT_KIND.Physical;
    const mockMetadataKind = NFT_METADATA_KIND.Raw;
    const mockOwnershipMode = NFT_OWNERSHIP_MODE.Transferable;
    const mockJsonSchema = '{"type": "object"}';

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url');
    });

    it('should return the correct collection name', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockCollectionName
      );
      const result = await client.collectionName();
      expect(result).toBe(mockCollectionName);
    });

    it('should return the correct collection symbol', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockCollectionSymbol
      );
      const result = await client.collectionSymbol();
      expect(result).toBe(mockCollectionSymbol);
    });

    it('should return the correct total token supply', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockTotalTokenSupply
      );
      const result = await client.tokenTotalSupply();
      expect(result).toBe(mockTotalTokenSupply);
    });

    it('should return the correct number of minted tokens', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockNumOfMintedTokens
      );
      const result = await client.numOfMintedTokens();
      expect(result).toBe(mockNumOfMintedTokens);
    });

    it('should return true when minting is allowed', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockAllowMinting
      );
      const result = await client.allowMinting();
      expect(result).toBe(true);
    });

    it('should return the correct minting mode', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockMintingMode
      );
      const result = await client.mintingMode();
      expect(result).toBe(MINTING_MODE[mockMintingMode]);
    });

    it('should return the correct whitelist mode', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockWhitelistMode
      );
      const result = await client.whitelistMode();
      expect(result).toBe(WHITELIST_MODE[mockWhitelistMode]);
    });

    it('should return the correct reporting mode', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockReportingMode
      );
      const result = await client.reportingMode();
      expect(result).toBe(OWNER_REVERSE_LOOKUP_MODE[mockReportingMode]);
    });

    it('should return the correct burn mode', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockBurnMode
      );
      const result = await client.burnMode();
      expect(result).toBe(BURN_MODE[mockBurnMode]);
    });

    it('should return the correct holder mode', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockHolderMode
      );
      const result = await client.holderMode();
      expect(result).toBe(NFT_HOLDER_MODE[mockHolderMode]);
    });

    it('should return the correct identifier mode', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockIdentifierMode
      );
      const result = await client.identifierMode();
      expect(result).toBe(NFT_IDENTIFIER_MODE[mockIdentifierMode]);
    });

    it('should return the correct metadata mutability', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockMetadataMutability
      );
      const result = await client.metadataMutability();
      expect(result).toBe(METADATA_MUTABILITY[mockMetadataMutability]);
    });

    it('should return the correct NFT kind', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockNftKind
      );
      const result = await client.nftKind();
      expect(result).toBe(NFT_KIND[mockNftKind]);
    });

    it('should return the correct metadata kind', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockMetadataKind
      );
      const result = await client.metadataKind();
      expect(result).toBe(NFT_METADATA_KIND[mockMetadataKind]);
    });

    it('should return the correct ownership mode', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockOwnershipMode
      );
      const result = await client.ownershipMode();
      expect(result).toBe(NFT_OWNERSHIP_MODE[mockOwnershipMode]);
    });

    it('should return the correct JSON schema', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockJsonSchema
      );
      const result = await client.jsonSchema();
      expect(result).toBe(mockJsonSchema);
    });

    it('should throw an error if queryContractData fails', async () => {
      vi.spyOn(client as any, 'queryContractData').mockRejectedValue(
        new Error('Query failed')
      );
      await expect(client.collectionName()).rejects.toThrow('Query failed');
    });
  });
});
