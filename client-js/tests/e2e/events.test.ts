import { CEP78Client, CEP78_EVENTS, CEP78EventResult } from 'src';
import { expect, describe, it, beforeEach } from 'vitest';
import { RPC_URL, SSE_URL, CHAIN_NAME } from '../../config';
import { getAccountInfo, findKeyFromAccountNamedKeys } from '../utils';
import { owner, install, mint } from './helpers';

let client: CEP78Client;
const collectionName = `TEST_CEP78_E2E_${Math.floor(Math.random() * 1000000)}`,
  waitForTransactionProcessed = false; // Do not wait for transactions execution, avoid duplicate listener of event

describe('CEP78Client - Event Streaming', () => {
  beforeEach(async () => {
    client = new CEP78Client(RPC_URL, SSE_URL, CHAIN_NAME);
    await install(client, collectionName);
    const account = await getAccountInfo(RPC_URL, owner.publicKey),
      contractHash = findKeyFromAccountNamedKeys(
        account,
        `cep78_contract_hash_${collectionName}`
      );
    expect(contractHash).toBeDefined();
    client.setContractHash(contractHash);
  }, 180000);

  it('should start and stop event stream and listen to events when on() is called', async () => {
    // Start the event stream
    client.startEventStream();

    const mintEvent = CEP78_EVENTS.Mint;
    let mintEventReceived = false;

    // Mint tokens and listen for the Mint event
    await mint(client, waitForTransactionProcessed);

    // Listen for Mint event in a promise awaited
    await new Promise<CEP78EventResult>((resolve) => {
      client.on(mintEvent, async (eventResult) => {
        mintEventReceived = true;
        resolve(eventResult);
      });
    });

    expect(mintEventReceived).toBe(true);

    // Stop the event stream
    client.stopEventStream();

    // Try to mint tokens again, but event listener shouldn't trigger
    await mint(client, waitForTransactionProcessed);

    let eventFired = false;
    client.on(mintEvent, () => {
      eventFired = true;
    });

    // The listener shouldn't fire because stream is stopped
    setTimeout(() => {
      expect(eventFired).toBe(false);
    }, 1000);
  }, 180000);

  it('should remove a specific event listener using off()', async () => {
    client.startEventStream();
    const mintEvent = CEP78_EVENTS.Mint;
    let eventTriggered = false;

    const listener = () => {
      eventTriggered = true;
    };

    client.on(mintEvent, listener);
    client.off(mintEvent, listener); // Remove the listener

    await mint(client, waitForTransactionProcessed);

    setTimeout(() => {
      expect(eventTriggered).toBe(false);
    }, 1000);

    client.stopEventStream();
  }, 180000);

  it('should remove all listeners for a specific event using removeListenersForEvent()', async () => {
    client.startEventStream();
    const mintEvent = CEP78_EVENTS.Mint;
    let firstListenerTriggered = false;
    let secondListenerTriggered = false;

    client.on(mintEvent, () => {
      firstListenerTriggered = true;
    });

    client.on(mintEvent, () => {
      secondListenerTriggered = true;
    });

    client.removeListenersForEvent(mintEvent); // Remove all listeners for Mint

    await mint(client, waitForTransactionProcessed);

    setTimeout(() => {
      expect(firstListenerTriggered).toBe(false);
      expect(secondListenerTriggered).toBe(false);
    }, 1000);

    client.stopEventStream();
  }, 180000);

  it('should remove all event listeners using removeAllListeners()', async () => {
    client.startEventStream();
    let mintTriggered = false;
    let burnTriggered = false;

    client.on(CEP78_EVENTS.Mint, () => {
      mintTriggered = true;
    });

    client.on(CEP78_EVENTS.Burn, () => {
      burnTriggered = true;
    });

    client.removeAllListeners(); // Remove all event listeners

    await mint(client, waitForTransactionProcessed);

    setTimeout(() => {
      expect(mintTriggered).toBe(false);
      expect(burnTriggered).toBe(false);
    }, 1000);

    client.stopEventStream();
  }, 180000);
});
