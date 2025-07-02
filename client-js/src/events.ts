import { CLValue, Hash, Message, TransactionHash } from 'casper-js-sdk';

export enum CEP47_EVENTS {
  Mint = 'Mint',
  Burn = 'Burn',
  Approval = 'Approval',
  ApprovalRevoked = 'ApprovalRevoked',
  ApprovalForAll = 'ApprovalForAll',
  RevokedForAll = 'RevokedForAll',
  Transfer = 'Transfer',
  MetadataUpdated = 'MetadataUpdated',
  VariablesSet = 'VariablesSet',
  Migration = 'Migration',
}

export enum CEP78_EVENTS {
  Mint = 'Mint',
  Burn = 'Burn',
  Approval = 'Approval',
  ApprovalRevoked = 'ApprovalRevoked',
  ApprovalForAll = 'ApprovalForAll',
  RevokedForAll = 'RevokedForAll',
  Transfer = 'Transfer',
  MetadataUpdated = 'MetadataUpdated',
  VariablesSet = 'VariablesSet',
  Migration = 'Migration',
}

type EventName = keyof typeof CEP78_EVENTS;

export type Event<E extends Record<string, CLValue>> = {
  name: EventName;
  contractHash: Hash;
  contractPackageHash: Hash;
  eventId: number;
  data: E;
};

export interface TransactionInfo {
  transactionHash: TransactionHash;
  timestamp: string;
  messages: Message[];
}

export type WithTransactionInfo<E> = E & { transactionInfo: TransactionInfo };

export type CEP78EventResult = WithTransactionInfo<CEP78Event>;

export type CEP78Event = Event<
  | Mint
  | Burn
  | Approval
  | ApprovalRevoked
  | ApprovalForAll
  | RevokedForAll
  | Transfer
  | MetadataUpdated
  | VariablesSet
  | Migration
>;

export type EventsMap = {
  Mint: WithTransactionInfo<Event<Mint>>;
  Burn: WithTransactionInfo<Event<Burn>>;
  Approval: WithTransactionInfo<Event<Approval>>;
  ApprovalRevoked: WithTransactionInfo<Event<ApprovalRevoked>>;
  ApprovalForAll: WithTransactionInfo<Event<ApprovalForAll>>;
  RevokedForAll: WithTransactionInfo<Event<RevokedForAll>>;
  Transfer: WithTransactionInfo<Event<Transfer>>;
  MetadataUpdated: WithTransactionInfo<Event<MetadataUpdated>>;
  VariablesSet: WithTransactionInfo<Event<VariablesSet>>;
  Migration: WithTransactionInfo<Event<Migration>>;
};

export type Mint = { recipient: CLValue; token_id: CLValue; data: CLValue };

export type Burn = { owner: CLValue; amount: CLValue };

export type Approval = { owner: CLValue; spender: CLValue; token_id: CLValue };

export type ApprovalRevoked = { owner: CLValue; token_id: CLValue };

export type ApprovalForAll = { owner: CLValue; operator: CLValue };

export type RevokedForAll = { owner: CLValue; operator: CLValue };

export type Transfer = { sender: CLValue; recipient: CLValue; amount: CLValue };

export type MetadataUpdated = { token_id: CLValue; data: CLValue };

export type VariablesSet = {};

export type Migration = {};
