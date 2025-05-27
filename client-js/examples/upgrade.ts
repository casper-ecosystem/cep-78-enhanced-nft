import { CHAIN_NAME, PRIVATE_KEY_FAUCET, RPC_URL, SSE_URL } from '../config';
import {
  CEP78Client,
  EVENTS_MODE,
  type TransactionParams,
  type TransactionResult,
  type UpgradeArgs,
} from '../dist';
import {
  findKeyFromAccountNamedKeys,
  getAccountInfo,
  getSigningKey,
} from '../tests/utils';

if (!PRIVATE_KEY_FAUCET) {
  throw new Error('FAUCET_SECRET_KEY environment variable is not set.');
}

const collectionName = 'TEST_CEP78',
  eventsMode = EVENTS_MODE.CES,
  waitForTransactionProcessed = true,
  sender = getSigningKey(PRIVATE_KEY_FAUCET),
  paymentAmount = String(600_000_000_000);

const upgrade = async () => {
  const cep78 = new CEP78Client(RPC_URL, SSE_URL, CHAIN_NAME),
    params: TransactionParams = {
      sender: sender.publicKey,
      paymentAmount,
      signingKeys: [sender],
    },
    args: UpgradeArgs = {
      collectionName,
      eventsMode,
    },
    transactionResult: TransactionResult = await cep78.upgrade({
      params,
      args,
      waitForTransactionProcessed,
    });

  if (!transactionResult.transactionInfo.transactionHash) {
    throw Error('Invalid transaction hash');
  }
  return transactionResult;
};

upgrade()
  .then(async (transactionResult) => {
    const { transactionInfo, executionResult } = transactionResult;
    console.info(
      `Contract upgrade transaction hash: ${transactionInfo.transactionHash.toHex()}`
    );

    if (executionResult) {
      if (executionResult?.errorMessage) {
        throw new Error(
          `Error during upgrade.\n${executionResult?.errorMessage.toString()}`
        );
      } else {
        console.info(
          `Contract upgrade cost consumed: ${executionResult?.consumed}`
        );
      }
    }

    const account = await getAccountInfo(RPC_URL, sender.publicKey),
      contractHash = findKeyFromAccountNamedKeys(
        account,
        `cep78_contract_hash_${collectionName}`
      ),
      contractPackageHash = findKeyFromAccountNamedKeys(
        account,
        `cep78_contract_package_${collectionName}`
      );

    console.info(`Contract Hash: ${contractHash}`);
    console.info(`Contract Package Hash: ${contractPackageHash}`);
  })
  .catch((error) => {
    console.error(error);
  });
