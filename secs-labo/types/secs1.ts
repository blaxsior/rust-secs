export type Secs1JsonString = string;
export type Secs1DebugString = string;
export type Secs1Bytes = Uint8Array;

export type Secs1DeviceId = number;
export type Secs1StreamId = number;
export type Secs1FunctionId = number;
export type Secs1SystemByte = number;
export type Secs1Duration = string;
export type Secs1ConnectionRole = "Active" | "Passive";
export type Secs1TimeoutUnit = "t1" | "t2";

export type Secs1TransportConfig = {
  device_id: Secs1DeviceId;
  local_role: Secs1ConnectionRole;
  t1_timeout: Secs1Duration;
  t2_timeout: Secs1Duration;
  t3_timeout: Secs1Duration;
  t4_timeout: Secs1Duration;
  t2_rty_limit: number;
};

export type Secs1BlockHeader = {
  device_id: Secs1DeviceId;
  rbit: boolean;
  wbit: boolean;
  stream: Secs1StreamId;
  function: Secs1FunctionId;
  ebit: boolean;
  block_no: number;
  system_byte: Secs1SystemByte;
};

export type Secs1Block = {
  header: Secs1BlockHeader;
  data: number[];
};

export type Secs1TimeoutKey = {
  id: number;
  unit: Secs1TimeoutUnit;
};

export type Secs1TransportConfigJson = Secs1JsonString;
export type Secs1BlockJson = Secs1JsonString;
export type Secs1TimeoutKeyJson = Secs1JsonString;
export type Secs1EventDebug = Secs1DebugString;

export interface JsSecs1BlockTransfer {
  read(bytes: Secs1Bytes): void;
  write(block_json: Secs1BlockJson): void;
  timeout(key: Secs1TimeoutKeyJson): void;
  poll_write(): Uint8Array | undefined;
  poll_read(): Secs1BlockJson | undefined;
  poll_timeout(): Secs1TimeoutKeyJson | undefined;
  poll_event(): Secs1EventDebug | undefined;
  free(): void;
}

export type JsSecs1BlockTransferConstructor = {
  new (config_json: Secs1TransportConfigJson): JsSecs1BlockTransfer;
};

export function stringify_secs1_config(config: Secs1TransportConfig): Secs1TransportConfigJson {
  return JSON.stringify(config);
}

export function stringify_secs1_block(block: Secs1Block): Secs1BlockJson {
  return JSON.stringify(block);
}

export function parse_secs1_block(json: Secs1BlockJson): Secs1Block {
  return JSON.parse(json) as Secs1Block;
}

export function parse_secs1_timeout_key(json: Secs1TimeoutKeyJson): Secs1TimeoutKey {
  return JSON.parse(json) as Secs1TimeoutKey;
}
