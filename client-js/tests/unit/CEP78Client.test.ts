/* eslint-disable @typescript-eslint/no-explicit-any */
import {
  Args,
  CLValue,
  ContractHash,
  ContractPackageHash,
  ExecutionResult,
  Key,
  KeyAlgorithm,
  ParamDictionaryIdentifierContractNamedKey,
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
  IDENTIFIER_MODE,
  NFT_METADATA_KIND,
  OWNERSHIP_MODE,
  BURN_MODE,
  MINTING_MODE,
  HOLDER_MODE,
  NFT_KIND,
  OWNER_REVERSE_LOOKUP_MODE,
  WHITELIST_MODE,
  RegisterParams,
  SetApprovallForAllParams,
  TokenMetadataParams,
  OwnerOfParams,
  StoreBalanceOfParams,
  GetApprovedParams,
  SetVariablesParams,
  OperatorArgs,
  StoreIsApprovedForAlldParams,
} from '../../src';

const mockTransactionHash = { toHex: () => 'mockTransactionHash' };

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
        ownershipMode: OWNERSHIP_MODE.Minter,
        nftMetadataKind: NFT_METADATA_KIND.CEP78,
        identifierMode: IDENTIFIER_MODE.Ordinal,
        metadataMutability: METADATA_MUTABILITY.Immutable,
        transferFilterContract: ContractHash.newContract(
          'hash-5eab221b01c32145051f47fa8c778b5a9ac5e01502d48dd13e5caa4973106906'
        ),
      },
      waitForTransactionProcessed: false,
    };

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url');
      vi.spyOn(client['_rpcClient'], 'putTransaction').mockResolvedValue({
        transactionHash: mockTransactionHash,
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionHash: mockTransactionHash,
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.mint(paramsWithWait);

      // Validate result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
    });

    it('should call callEntrypoint with correct runtime arguments', async () => {
      await client.burn(mockParams);

      // Retrieve runtimeArgs from spy call
      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate runtime arguments for burn
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          token_hash: CLValue.newCLString(mockParams.args.tokenHash!),
        })
      );
    });

    it('should successfully execute burn when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.burn(paramsWithWait);

      // Validate result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.transfer(paramsWithWait);

      // Check the result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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

  describe('CEP78Client - register', () => {
    let client: CEP78Client;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const ownerKey = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockParams: RegisterParams = {
      params: {
        sender: key.publicKey,
        paymentAmount: '1000',
        signingKeys: [key],
        chainName: 'testnet',
      },
      args: {
        tokenOwner: ownerKey.publicKey,
      },
      waitForTransactionProcessed: false,
    };

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url');
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
      vi.spyOn(client, 'waitForTransactionProcessed').mockResolvedValue({
        transactionProcessedPayload: {
          executionResult: { errorMessage: '' } as ExecutionResult,
        } as unknown as TransactionProcessedPayload,
      });
    });

    it('should successfully register a token owner', async () => {
      const result = await client.register(mockParams);

      // Verify callEntrypoint was called with correct parameters
      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'register_owner',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        mockParams.waitForTransactionProcessed
      );

      // Validate result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
    });

    it('should call callEntrypoint with correct runtime arguments for register', async () => {
      await client.register(mockParams);

      // Retrieve runtimeArgs from spy call
      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate runtime arguments for register
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          token_owner: CLValue.newCLKey(
            Key.newKey(ownerKey.publicKey.accountHash().toPrefixedString())
          ),
        })
      );
    });

    it('should successfully execute register when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.register(paramsWithWait);

      // Validate result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });
    });

    it('should handle the case when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      await client.register(paramsWithWait);

      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'register_owner',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        true
      );
    });

    it('should handle errors during the register process', async () => {
      const errorMessage = 'Error during register.';
      vi.spyOn(client as any, 'callEntrypoint').mockRejectedValueOnce(
        new Error(errorMessage)
      );

      await expect(client.register(mockParams)).rejects.toThrow(
        'Error during register.'
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.approve(paramsWithWait);

      // Validate result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
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

  describe('CEP78Client - revoke', () => {
    let client: CEP78Client;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const operatorKey = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockParams: ApproveParams = {
      params: {
        sender: key.publicKey,
        paymentAmount: '1000',
        signingKeys: [key],
        chainName: 'testnet',
      },
      args: {
        operator: operatorKey.publicKey,
        tokenHash: 'tokenHash',
        tokenId: undefined, // Can be either tokenHash or tokenId, not both
      },
      waitForTransactionProcessed: false,
    };

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url');
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
      vi.spyOn(client, 'waitForTransactionProcessed').mockResolvedValue({
        transactionProcessedPayload: {
          executionResult: { errorMessage: '' } as ExecutionResult,
        } as unknown as TransactionProcessedPayload,
      });
    });

    it('should successfully revoke', async () => {
      const result = await client.revoke(mockParams);

      // Verify callEntrypoint was called with correct parameters
      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'revoke',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        mockParams.waitForTransactionProcessed
      );

      // Validate result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
    });

    it('should call callEntrypoint with correct runtime arguments', async () => {
      await client.revoke(mockParams);

      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate runtime arguments for revoke
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          operator: CLValue.newCLKey(
            Key.newKey(operatorKey.publicKey.accountHash().toPrefixedString())
          ),
          token_hash: CLValue.newCLString('tokenHash'),
        })
      );
    });

    it('should successfully revoke when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.revoke(paramsWithWait);

      // Validate result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });
    });

    it('should handle the case when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      await client.revoke(paramsWithWait);

      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'revoke',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        true
      );
    });

    it('should handle errors during the revoke process', async () => {
      const errorMessage = 'Error during revoke.';
      vi.spyOn(client as any, 'callEntrypoint').mockRejectedValueOnce(
        new Error(errorMessage)
      );

      await expect(client.revoke(mockParams)).rejects.toThrow(
        'Error during revoke.'
      );
    });

    it('should insert token_id if provided', async () => {
      const mockParamsWithTokenId: ApproveParams = {
        ...mockParams,
        args: {
          operator: operatorKey.publicKey,
          tokenId: '12345', // Providing tokenId instead of tokenHash
        },
      };

      await client.revoke(mockParamsWithTokenId);

      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate token_id is included in the runtime arguments
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          operator: CLValue.newCLKey(
            Key.newKey(operatorKey.publicKey.accountHash().toPrefixedString())
          ),
          token_id: CLValue.newCLUint64(12345),
        })
      );
    });

    it('should insert token_hash if provided', async () => {
      const mockParamsWithTokenHash: ApproveParams = {
        ...mockParams,
        args: {
          operator: operatorKey.publicKey,
          tokenHash: 'tokenHash', // Providing tokenHash
        },
      };

      await client.revoke(mockParamsWithTokenHash);

      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate token_hash is included in the runtime arguments
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          operator: CLValue.newCLKey(
            Key.newKey(operatorKey.publicKey.accountHash().toPrefixedString())
          ),
          token_hash: CLValue.newCLString('tokenHash'),
        })
      );
    });
  });

  describe('CEP78Client - setApprovalForAll', () => {
    let client: CEP78Client;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const operatorKey = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockParams: SetApprovallForAllParams = {
      params: {
        sender: key.publicKey,
        paymentAmount: '1000',
        signingKeys: [key],
        chainName: 'testnet',
      },
      args: {
        operator: operatorKey.publicKey,
        approveAll: true,
      },
      waitForTransactionProcessed: false,
    };

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url');
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
      vi.spyOn(client, 'waitForTransactionProcessed').mockResolvedValue({
        transactionProcessedPayload: {
          executionResult: { errorMessage: '' } as ExecutionResult,
        } as unknown as TransactionProcessedPayload,
      });
    });

    it('should successfully set approval for all', async () => {
      const result = await client.setApprovalForAll(mockParams);

      // Verify callEntrypoint was called with correct parameters
      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'set_approval_for_all',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        mockParams.waitForTransactionProcessed
      );

      // Validate result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
    });

    it('should call callEntrypoint with correct runtime arguments', async () => {
      await client.setApprovalForAll(mockParams);

      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate runtime arguments for setApprovalForAll
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          approve_all: CLValue.newCLValueBool(true),
          operator: CLValue.newCLKey(
            Key.newKey(operatorKey.publicKey.accountHash().toPrefixedString())
          ),
        })
      );
    });

    it('should successfully set approval for all when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.setApprovalForAll(paramsWithWait);

      // Validate result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });
    });

    it('should handle the case when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      await client.setApprovalForAll(paramsWithWait);

      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'set_approval_for_all',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        true
      );
    });

    it('should handle errors during the setApprovalForAll process', async () => {
      const errorMessage = 'Error during set approval for all.';
      vi.spyOn(client as any, 'callEntrypoint').mockRejectedValueOnce(
        new Error(errorMessage)
      );

      await expect(client.setApprovalForAll(mockParams)).rejects.toThrow(
        'Error during set approval for all.'
      );
    });

    it('should handle approveAll as false correctly', async () => {
      const mockParamsWithApproveAllFalse: SetApprovallForAllParams = {
        ...mockParams,
        args: {
          operator: operatorKey.publicKey,
          approveAll: false, // Setting approveAll to false
        },
      };

      await client.setApprovalForAll(mockParamsWithApproveAllFalse);

      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate approve_all is set to false in the runtime arguments
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          approve_all: CLValue.newCLValueBool(false),
          operator: CLValue.newCLKey(
            Key.newKey(operatorKey.publicKey.accountHash().toPrefixedString())
          ),
        })
      );
    });
  });

  describe('CEP78Client - setTokenMetadata', () => {
    let client: CEP78Client;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockParams: TokenMetadataParams = {
      params: {
        sender: key.publicKey,
        paymentAmount: '1000',
        signingKeys: [key],
        chainName: 'testnet',
      },
      args: {
        tokenMetaData: {
          name: 'Test Token',
          description: 'A test token metadata',
        },
      },
      waitForTransactionProcessed: false,
    };

    beforeEach(() => {
      client = new CEP78Client('http://mock-rpc-url');
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
      vi.spyOn(client, 'waitForTransactionProcessed').mockResolvedValue({
        transactionProcessedPayload: {
          executionResult: { errorMessage: '' } as ExecutionResult,
        } as unknown as TransactionProcessedPayload,
      });
    });

    it('should successfully set token metadata', async () => {
      const result = await client.setTokenMetadata(mockParams);

      // Verify callEntrypoint was called with correct parameters
      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'set_token_metadata',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        mockParams.waitForTransactionProcessed
      );

      // Validate result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
    });

    it('should call callEntrypoint with correct runtime arguments', async () => {
      await client.setTokenMetadata(mockParams);

      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate runtime arguments for setTokenMetadata
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          token_meta_data: CLValue.newCLString(
            JSON.stringify(mockParams.args.tokenMetaData)
          ),
        })
      );
    });

    it('should successfully set token metadata when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });

      const result = await client.setTokenMetadata(paramsWithWait);

      // Validate result
      expect(result).toEqual({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
        executionResult: { errorMessage: '' } as ExecutionResult,
      });
    });

    it('should handle the case when waitForTransactionProcessed is true', async () => {
      const paramsWithWait = {
        ...mockParams,
        waitForTransactionProcessed: true,
      };
      await client.setTokenMetadata(paramsWithWait);

      expect(client['callEntrypoint']).toHaveBeenCalledWith(
        'set_token_metadata',
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        true
      );
    });

    it('should handle errors during the setTokenMetadata process', async () => {
      const errorMessage = 'Error during setting token metadata.';
      vi.spyOn(client as any, 'callEntrypoint').mockRejectedValueOnce(
        new Error(errorMessage)
      );

      await expect(client.setTokenMetadata(mockParams)).rejects.toThrow(
        'Error during setting token metadata.'
      );
    });

    it('should handle empty tokenMetaData correctly', async () => {
      const mockParamsWithEmptyMetaData: TokenMetadataParams = {
        ...mockParams,
        args: {
          tokenMetaData: {}, // Empty metadata
        },
      };

      await client.setTokenMetadata(mockParamsWithEmptyMetaData);

      const runtimeArgs = (client as any).callEntrypoint.mock.calls[0][1];

      // Validate that empty metadata is handled correctly
      expect(runtimeArgs).toEqual(
        Args.fromMap({
          token_meta_data: CLValue.newCLString(
            JSON.stringify(mockParamsWithEmptyMetaData.args.tokenMetaData)
          ),
        })
      );
    });
  });

  describe('CEP78Client - ownerOf', () => {
    let client: CEP78Client;
    let mockRpcClient: RpcClient;
    const mockKey = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockParams: OwnerOfParams = {
      params: {
        wasm: new Uint8Array(),
        sender: mockKey.publicKey,
        paymentAmount: '1000',
        signingKeys: [mockKey],
        chainName: 'testnet',
      },
      args: {
        tokenId: '1',
        keyName: 'mockKeyName',
      },
      waitForTransactionProcessed: false,
    };

    const contractHash =
      'hash-a84b9f15e57097579cb651bc3eec5143972c8c9ea153bb26d07367f9d41a767b';

    beforeEach(() => {
      mockRpcClient = {
        getDictionaryItemByIdentifier: vi.fn(),
      } as unknown as RpcClient;
      client = new CEP78Client('http://mock-rpc-url').setContractHash(
        contractHash
      );
      client['_rpcClient'] = mockRpcClient;

      vi.spyOn(client as any, 'callSession').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValue({
        storedValue: {
          clValue: {
            toString: () => 'mockOwnerAddress',
          },
        },
      } as StateGetDictionaryResult);
    });

    // afterEach(() => {
    //   vi.restoreAllMocks();
    // });

    it('should return the owner address when owner exists', async () => {
      const result = await client.ownerOf('1');

      // Verifying that the contract hash is used
      const expectedKey = contractHash;
      const contractNamedKey = new ParamDictionaryIdentifierContractNamedKey(
        expectedKey,
        'token_owners',
        '1'
      );

      // Check that the dictionary state was retrieved correctly
      expect(mockRpcClient.getDictionaryItemByIdentifier).toHaveBeenCalledWith(
        null,
        expect.objectContaining({
          contractNamedKey,
        })
      );

      // Check if the correct owner address is returned
      expect(result).toBe('mockOwnerAddress');
    });

    it('should call callSession with correct arguments when keyName is provided', async () => {
      await client.ownerOf(mockParams);

      expect(client['callSession']).toHaveBeenCalledWith(
        new Uint8Array(),
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        mockParams.waitForTransactionProcessed
      );
    });

    it('should throw error when contract hash is not set', async () => {
      (client as any)['_contractHash'] = undefined;

      await expect(client.ownerOf(mockParams)).rejects.toThrowError(
        'Contract hash is not set.'
      );
    });

    it('should return undefined if no owner found in dictionary', async () => {
      const consoleWarnSpy = vi
        .spyOn(console, 'warn')
        .mockImplementation(() => {});
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockRejectedValueOnce(new Error('Query failed'));

      const result = await client.ownerOf('mockTokenId');
      expect(result).toBeUndefined();
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        'No owner found for mockTokenId'
      );
    });

    it('should return undefined if the identifier is not found', async () => {
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValueOnce({
        storedValue: { clValue: undefined },
      } as StateGetDictionaryResult);

      const result = await client.ownerOf('mockTokenId');
      expect(result).toBeUndefined();
    });

    it('should handle error in getDictionaryItemByIdentifier gracefully', async () => {
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockRejectedValueOnce(new Error('Other error'));

      await expect(client.ownerOf('mockTokenId')).rejects.toThrowError(
        'Other error'
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
      client.setContractHash(contractHash);
      client['_rpcClient'] = mockRpcClient;

      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValue({
        storedValue: {
          clValue: {
            toString: () => mockBalance,
          },
        },
      } as StateGetDictionaryResult);
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it('should return the balance as a string when found', async () => {
      const balance = await client.balanceOf(key.publicKey);
      expect(balance).toBe(mockBalance);
    });

    it('should call callSession with correct arguments when keyName is provided', async () => {
      const mockParams: StoreBalanceOfParams = {
        params: {
          wasm: new Uint8Array(),
          sender: key.publicKey,
          paymentAmount: '1000',
          signingKeys: [key],
          chainName: 'testnet',
        },
        args: {
          tokenOwner: key.publicKey,
          keyName: 'mockKeyName',
        },
        waitForTransactionProcessed: false,
      };

      vi.spyOn(client as any, 'callSession').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });

      await client.balanceOf(mockParams);

      expect(client['callSession']).toHaveBeenCalledWith(
        new Uint8Array(),
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        mockParams.waitForTransactionProcessed
      );
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

  describe('CEP78Client - getApproved', () => {
    let client: CEP78Client;
    let mockRpcClient: RpcClient;
    const key = PrivateKey.generate(KeyAlgorithm.ED25519);
    const contractHash =
      'hash-a84b9f15e57097579cb651bc3eec5143972c8c9ea153bb26d07367f9d41a767b';
    const mockTokenId = '1';
    const mockTokenHash = 'mockTokenHash';
    const mockKeyName = 'mockKeyName';
    const mockApproval = 'mockApprovalAddress';

    beforeEach(() => {
      mockRpcClient = {
        getDictionaryItemByIdentifier: vi.fn(),
      } as unknown as RpcClient;

      client = new CEP78Client('http://mock-rpc-url');
      client.setContractHash(contractHash);
      client['_rpcClient'] = mockRpcClient;

      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValue({
        storedValue: {
          clValue: {
            toString: () => mockApproval,
          },
        },
      } as StateGetDictionaryResult);
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it('should return the approval address when approval exists', async () => {
      const result = await client.getApproved(mockTokenId);

      // Verifying that the contract hash is used
      const expectedKey = contractHash;
      const contractNamedKey = new ParamDictionaryIdentifierContractNamedKey(
        expectedKey,
        'approved',
        mockTokenId
      );

      // Check that the dictionary state was retrieved correctly
      expect(mockRpcClient.getDictionaryItemByIdentifier).toHaveBeenCalledWith(
        null,
        expect.objectContaining({
          contractNamedKey,
        })
      );

      // Check if the correct approval address is returned
      expect(result).toBe(mockApproval);
    });

    it('should call callSession with correct arguments when keyName is provided', async () => {
      const mockParams: GetApprovedParams = {
        params: {
          wasm: new Uint8Array(),
          sender: key.publicKey,
          paymentAmount: '1000',
          signingKeys: [key],
          chainName: 'testnet',
        },
        args: {
          tokenId: mockTokenId,
          tokenHash: mockTokenHash,
          keyName: mockKeyName,
        },
        waitForTransactionProcessed: false,
      };

      vi.spyOn(client as any, 'callSession').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });

      await client.getApproved(mockParams);

      expect(client['callSession']).toHaveBeenCalledWith(
        new Uint8Array(),
        expect.anything(),
        mockParams.params.paymentAmount,
        mockParams.params.sender,
        mockParams.params.signingKeys,
        mockParams.params.chainName,
        mockParams.waitForTransactionProcessed
      );
    });

    it('should throw an error when contract hash is not set', async () => {
      const clientWithoutContractHash = new CEP78Client('http://mock-rpc-url');
      await expect(
        clientWithoutContractHash.getApproved(mockTokenId)
      ).rejects.toThrowError('Contract hash is not set.');
    });

    it('should return empty string when approval not found in dictionary', async () => {
      const consoleWarnSpy = vi
        .spyOn(console, 'warn')
        .mockImplementation(() => {});
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockRejectedValueOnce(new Error('Query failed'));

      const result = await client.getApproved(mockTokenId);
      expect(result).toBe('');
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        `No approval found for ${mockTokenId}`
      );
    });

    it('should return undefined if the identifier is not found', async () => {
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValueOnce({
        storedValue: { clValue: undefined },
      } as StateGetDictionaryResult);

      const result = await client.getApproved(mockTokenId);
      expect(result).toBeUndefined();
    });

    it('should handle error in getDictionaryItemByIdentifier gracefully', async () => {
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockRejectedValueOnce(new Error('Other error'));

      await expect(client.getApproved(mockTokenId)).rejects.toThrowError(
        'Other error'
      );
    });

    it('should construct the correct dictionary key when using tokenId', async () => {
      await client.getApproved(mockTokenId);

      const dictionaryIdentifier = (mockRpcClient as any)
        .getDictionaryItemByIdentifier.mock.calls[0][1];

      expect(dictionaryIdentifier.contractNamedKey).toEqual({
        key: contractHash,
        dictionaryName: 'approved',
        dictionaryItemKey: mockTokenId,
      });
    });

    it('should construct the correct dictionary key when using tokenHash', async () => {
      await client.getApproved(mockTokenHash);

      const dictionaryIdentifier = (mockRpcClient as any)
        .getDictionaryItemByIdentifier.mock.calls[0][1];

      expect(dictionaryIdentifier.contractNamedKey).toEqual({
        key: contractHash,
        dictionaryName: 'approved',
        dictionaryItemKey: mockTokenHash,
      });
    });
  });

  describe('CEP78Client - isApprovedForAll', () => {
    let client: CEP78Client;
    let mockRpcClient: RpcClient;
    const mockKey = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockPublicKey = mockKey.publicKey;
    const contractHash =
      'hash-a84b9f15e57097579cb651bc3eec5143972c8c9ea153bb26d07367f9d41a767b';

    beforeEach(() => {
      mockRpcClient = {
        getDictionaryItemByIdentifier: vi.fn(),
      } as unknown as RpcClient;

      client = new CEP78Client('http://mock-rpc-url');
      client.setContractHash(contractHash);
      client['_rpcClient'] = mockRpcClient;

      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValue({
        storedValue: {
          clValue: {
            toString: () => 'true',
          },
        },
      } as StateGetDictionaryResult);
    });

    it('should return true when approval is found in dictionary (OperatorArgs)', async () => {
      const operatorParams: OperatorArgs = {
        tokenOwner: mockPublicKey,
        operator: mockPublicKey,
      };

      const result = await client.isApprovedForAll(operatorParams);

      // Verifying that the dictionary state was retrieved correctly
      expect(mockRpcClient.getDictionaryItemByIdentifier).toHaveBeenCalledWith(
        null,
        expect.objectContaining({
          contractNamedKey: expect.objectContaining({
            key: contractHash,
            dictionaryName: 'operators',
            dictionaryItemKey: expect.any(String),
          }),
        })
      );

      // Check if the correct approval status is returned
      expect(result).toBe(true);
    });

    it('should return false when approval is not found in dictionary (OperatorArgs)', async () => {
      // Mocking no approval found scenario
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValueOnce({
        storedValue: {
          clValue: {
            toString: () => 'false',
          },
        },
      } as StateGetDictionaryResult);

      const operatorParams: OperatorArgs = {
        tokenOwner: mockPublicKey,
        operator: mockPublicKey,
      };

      const result = await client.isApprovedForAll(operatorParams);

      // Verifying that the dictionary state was retrieved correctly
      expect(mockRpcClient.getDictionaryItemByIdentifier).toHaveBeenCalledWith(
        null,
        expect.objectContaining({
          contractNamedKey: expect.objectContaining({
            key: contractHash,
            dictionaryName: 'operators',
            dictionaryItemKey: expect.any(String),
          }),
        })
      );

      // Check if the correct approval status is returned
      expect(result).toBe(false);
    });

    it('should throw an error if contract hash is not set', async () => {
      (client as any)['_contractHash'] = undefined;

      const operatorParams: OperatorArgs = {
        tokenOwner: mockPublicKey,
        operator: mockPublicKey,
      };

      try {
        await client.isApprovedForAll(operatorParams);
      } catch (error) {
        expect(error).toEqual(new Error('Contract hash is not set.'));
      }
    });

    it('should correctly handle StoreIsApprovedForAlldParams (WASM + keyName)', async () => {
      vi.spyOn(client as any, 'callSession').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });

      const storeParams: StoreIsApprovedForAlldParams = {
        params: {
          wasm: new Uint8Array(),
          sender: mockPublicKey,
          paymentAmount: '1000',
          signingKeys: [mockKey],
          chainName: 'testnet',
        },
        args: {
          tokenOwner: mockPublicKey,
          operator: mockPublicKey,
          keyName: 'mockKeyName',
        },
        waitForTransactionProcessed: false,
      };

      await client.isApprovedForAll(storeParams);

      // Verifying that the callSession method is called with correct params
      const argsMap = new Map([
        ['nft_contract_hash', expect.any(String)],
        ['token_owner', expect.any(String)],
        ['operator', expect.any(String)],
        ['key_name', 'mockKeyName'],
      ]);

      // Check if the args Map contains the expected data
      expect(client['callSession']).toHaveBeenCalledWith(
        expect.any(Uint8Array),
        expect.objectContaining({
          args: expect.objectContaining(argsMap),
        }),
        storeParams.params.paymentAmount,
        storeParams.params.sender,
        storeParams.params.signingKeys,
        storeParams.params.chainName,
        storeParams.waitForTransactionProcessed
      );
    });
  });

  describe('CEP78Client - isAclWhitelisted', () => {
    let client: CEP78Client;
    let mockRpcClient: RpcClient;
    const mockKey = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockPublicKey = mockKey.publicKey;
    const contractHash =
      'hash-a84b9f15e57097579cb651bc3eec5143972c8c9ea153bb26d07367f9d41a767b';

    beforeEach(() => {
      mockRpcClient = {
        getDictionaryItemByIdentifier: vi.fn(),
      } as unknown as RpcClient;

      client = new CEP78Client('http://mock-rpc-url');
      client.setContractHash(contractHash);
      client['_rpcClient'] = mockRpcClient;

      vi.spyOn(client as any, 'callSession').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValue({
        storedValue: {
          clValue: {
            toString: () => 'true',
          },
        },
      } as StateGetDictionaryResult);
    });

    it('should return true when the public key is whitelisted in the ACL', async () => {
      const params = mockPublicKey;

      const result = await client.isAclWhitelisted(params);

      // Verifying that the dictionary state was retrieved correctly
      expect(mockRpcClient.getDictionaryItemByIdentifier).toHaveBeenCalledWith(
        null,
        expect.objectContaining({
          contractNamedKey: expect.objectContaining({
            key: contractHash,
            dictionaryName: 'acl_whitelist',
            dictionaryItemKey: expect.any(String),
          }),
        })
      );

      // Check if the correct whitelisted status is returned
      expect(result).toBe(true);
    });

    it('should return false if the public key is not whitelisted', async () => {
      const consoleWarnSpy = vi
        .spyOn(console, 'warn')
        .mockImplementation(() => {});
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockRejectedValueOnce(new Error('Query failed'));

      const params = mockPublicKey;
      const result = await client.isAclWhitelisted(params);

      expect(result).toBe(false);
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        `No whiteListing for ${mockPublicKey.accountHash().toPrefixedString()}`
      );
    });

    it('should return false if the whitelist status is not true', async () => {
      const params = mockPublicKey;

      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValueOnce({
        storedValue: { clValue: { toString: () => 'false' } },
      } as StateGetDictionaryResult);

      const result = await client.isAclWhitelisted(params);

      expect(result).toBe(false);
    });

    it('should throw an error when contract hash is not set', async () => {
      const clientWithoutContractHash = new CEP78Client('http://mock-rpc-url');
      await expect(
        clientWithoutContractHash.isAclWhitelisted(mockPublicKey)
      ).rejects.toThrow('Contract hash is not set.');
    });

    it('should handle error gracefully when getDictionaryItemByIdentifier fails', async () => {
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockRejectedValueOnce(new Error('RPC Error'));

      const params = mockPublicKey;
      await expect(client.isAclWhitelisted(params)).rejects.toThrowError(
        'RPC Error'
      );
    });

    it('should construct the correct dictionary key when public key is provided', async () => {
      const params = mockPublicKey;
      const dictionaryItemKey = client['getPrefixedString'](mockPublicKey)
        .toPrefixedString()
        .replace(/^.*-/, '');

      await client.isAclWhitelisted(params);

      expect(mockRpcClient.getDictionaryItemByIdentifier).toHaveBeenCalledWith(
        null,
        expect.objectContaining({
          contractNamedKey: expect.objectContaining({
            key: contractHash,
            dictionaryName: 'acl_whitelist',
            dictionaryItemKey: dictionaryItemKey,
          }),
        })
      );
    });
  });

  describe('CEP78Client - setVariables', () => {
    let client: CEP78Client;
    let mockRpcClient: RpcClient;
    const mockKey = PrivateKey.generate(KeyAlgorithm.ED25519);
    const mockPublicKey = mockKey.publicKey;
    const contractHash =
      'hash-a84b9f15e57097579cb651bc3eec5143972c8c9ea153bb26d07367f9d41a767b';

    beforeEach(() => {
      mockRpcClient = {
        callEntrypoint: vi.fn(),
      } as unknown as RpcClient;

      client = new CEP78Client('http://mock-rpc-url');
      client.setContractHash(contractHash);
      client['_rpcClient'] = mockRpcClient;

      // Mocking callEntrypoint method directly on client
      vi.spyOn(client as any, 'callEntrypoint').mockResolvedValue({
        transactionInfo: {
          transactionHash: mockTransactionHash,
        },
      });
    });

    it('should call callEntrypoint with correct parameters when all parameters are provided', async () => {
      const setVariablesParams: SetVariablesParams = {
        params: {
          sender: mockPublicKey,
          paymentAmount: '1000',
          signingKeys: [mockKey],
          chainName: 'testnet',
        },
        args: {
          allowMinting: true,
          aclWhitelist: [mockPublicKey],
          aclPackageMode: false,
          packageOperatorMode: true,
          operatorBurnMode: false,
        },
        waitForTransactionProcessed: true,
      };

      await client.setVariables(setVariablesParams);

      // Verifying that callEntrypoint is called
      expect(client['callEntrypoint']).toHaveBeenCalled();
    });

    it('should call callEntrypoint when no optional parameters are provided', async () => {
      const setVariablesParams: SetVariablesParams = {
        params: {
          sender: mockPublicKey,
          paymentAmount: '1000',
          signingKeys: [mockKey],
          chainName: 'testnet',
        },
        args: {},
        waitForTransactionProcessed: true,
      };

      await client.setVariables(setVariablesParams);

      // Verifying that callEntrypoint is called even if no arguments were provided
      expect(client['callEntrypoint']).toHaveBeenCalled();
    });

    it('should call callEntrypoint with correct parameters when only allowMinting is provided', async () => {
      const setVariablesParams: SetVariablesParams = {
        params: {
          sender: mockPublicKey,
          paymentAmount: '1000',
          signingKeys: [mockKey],
          chainName: 'testnet',
        },
        args: {
          allowMinting: true,
        },
        waitForTransactionProcessed: true,
      };

      await client.setVariables(setVariablesParams);

      // Verifying that callEntrypoint is called when only allowMinting is provided
      expect(client['callEntrypoint']).toHaveBeenCalled();
    });

    it('should call callEntrypoint when aclWhitelist is not provided', async () => {
      const setVariablesParams: SetVariablesParams = {
        params: {
          sender: mockPublicKey,
          paymentAmount: '1000',
          signingKeys: [mockKey],
          chainName: 'testnet',
        },
        args: {},
        waitForTransactionProcessed: true,
      };

      await client.setVariables(setVariablesParams);

      // Verifying that callEntrypoint is called when no aclWhitelist is provided
      expect(client['callEntrypoint']).toHaveBeenCalled();
    });
  });

  describe('CEP78Client - metadata', () => {
    let client: CEP78Client;
    let mockRpcClient: RpcClient;
    const contractHash =
      'hash-a84b9f15e57097579cb651bc3eec5143972c8c9ea153bb26d07367f9d41a767b';
    const mockTokenIdentifier = 'mockTokenIdentifier';

    beforeEach(() => {
      mockRpcClient = {
        getDictionaryItemByIdentifier: vi.fn(),
      } as unknown as RpcClient;

      client = new CEP78Client('http://mock-rpc-url');
      client.setContractHash(contractHash);
      client['_rpcClient'] = mockRpcClient;

      // Mocking the return value for the metadata call
      vi.spyOn(client as any, 'nftMetadataKind').mockResolvedValue('CEP78'); // Mocking metadataKind method
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockResolvedValue({
        storedValue: { clValue: { toJSON: () => ({ key: 'mockData' }) } },
      } as StateGetDictionaryResult);
    });

    it('should call getDictionaryItemByIdentifier and return metadata for a given tokenIdentifier', async () => {
      const metadata = await client.metadata(mockTokenIdentifier);

      // Verifying that getDictionaryItemByIdentifier was called with the correct parameters
      expect(mockRpcClient.getDictionaryItemByIdentifier).toHaveBeenCalledWith(
        null,
        expect.objectContaining({
          contractNamedKey: expect.objectContaining({
            key: contractHash,
            dictionaryItemKey: mockTokenIdentifier,
          }),
        })
      );

      expect(metadata).toEqual({ key: 'mockData' });
    });

    it('should return an empty object when metadata is not found', async () => {
      // Mocking a failed response (Query failed error)
      vi.spyOn(
        mockRpcClient,
        'getDictionaryItemByIdentifier'
      ).mockRejectedValueOnce(new Error('Query failed'));

      const metadata = await client.metadata(mockTokenIdentifier);

      expect(metadata).toEqual({});
    });

    it('should throw an error if contract hash is not set', async () => {
      (client as any)['_contractHash'] = undefined;

      try {
        await client.metadata(mockTokenIdentifier);
      } catch (error) {
        expect(error).toEqual(new Error('Contract hash is not set.'));
      }
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
    const mockHolderMode = HOLDER_MODE.Mixed;
    const mockIdentifierMode = IDENTIFIER_MODE.Hash;
    const mockMetadataMutability = METADATA_MUTABILITY.Immutable;
    const mockNftKind = NFT_KIND.Physical;
    const mockNftMetadataKind = NFT_METADATA_KIND.Raw;
    const mockOwnershipMode = OWNERSHIP_MODE.Transferable;
    const mockeventsMode = EVENTS_MODE.CES;
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
      expect(result).toBe(HOLDER_MODE[mockHolderMode]);
    });

    it('should return the correct identifier mode', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockIdentifierMode
      );
      const result = await client.identifierMode();
      expect(result).toBe(IDENTIFIER_MODE[mockIdentifierMode]);
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
        mockNftMetadataKind
      );
      const result = await client.nftMetadataKind();
      expect(result).toBe(NFT_METADATA_KIND[mockNftMetadataKind]);
    });

    it('should return the correct ownership mode', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockOwnershipMode
      );
      const result = await client.ownershipMode();
      expect(result).toBe(OWNERSHIP_MODE[mockOwnershipMode]);
    });

    it('should return the correct events mode', async () => {
      vi.spyOn(client as any, 'queryContractData').mockResolvedValue(
        mockeventsMode
      );
      const result = await client.eventsMode();
      expect(result).toBe(EVENTS_MODE[mockeventsMode]);
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
