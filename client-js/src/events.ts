import { CLValue, Hash, Message } from 'casper-js-sdk';

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
  transactionHash: string;
  timestamp: string;
  messages: Message[];
}

export type WithTransactionInfo<E> = E & { transactionInfo: TransactionInfo };

export type CEP78EventResult = WithTransactionInfo<CEP78Event>;

export type CEP78Event = Event<
  | Mint
  | Burn
  | SetAllowance
  | IncreaseAllowance
  | DecreaseAllowance
  | Transfer
  | TransferFrom
>;

export type EventsMap = {
  Mint: WithTransactionInfo<Event<Mint>>;
  Burn: WithTransactionInfo<Event<Burn>>;
  SetAllowance: WithTransactionInfo<Event<SetAllowance>>;
  IncreaseAllowance: WithTransactionInfo<Event<IncreaseAllowance>>;
  DecreaseAllowance: WithTransactionInfo<Event<DecreaseAllowance>>;
  Transfer: WithTransactionInfo<Event<Transfer>>;
  TransferFrom: WithTransactionInfo<Event<TransferFrom>>;
};

export type Mint = { recipient: CLValue; amount: CLValue };

export type Burn = { owner: CLValue; amount: CLValue };

export type SetAllowance = {
  owner: CLValue;
  spender: CLValue;
  allowance: CLValue;
};

export type IncreaseAllowance = {
  owner: CLValue;
  spender: CLValue;
  allowance: CLValue;
  inc_by: CLValue;
};

export type DecreaseAllowance = {
  owner: CLValue;
  spender: CLValue;
  allowance: CLValue;
  decr_by: CLValue;
};

export type Transfer = { sender: CLValue; recipient: CLValue; amount: CLValue };

export type TransferFrom = {
  spender: CLValue;
  owner: CLValue;
  recipient: CLValue;
  amount: CLValue;
};
