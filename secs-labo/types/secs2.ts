export type Secs2StreamId = number;
export type Secs2FunctionId = number;

export type Secs2Message = {
  stream: Secs2StreamId;
  function: Secs2FunctionId;
  need_reply: boolean;
  body?: Secs2Item | null;
};

export type Secs2Item = Secs2Variant;

export type Secs2Variant =
  | { format: "list"; value: Secs2List }
  | { format: "binary"; value: Secs2Binary }
  | { format: "boolean"; value: Secs2Boolean }
  | { format: "ascii"; value: Secs2ASCII }
  // | { format: "jis8" }
  // | { format: "char" }
  | { format: "int8"; value: Secs2Int8 }
  | { format: "int1"; value: Secs2Int1 }
  | { format: "int2"; value: Secs2Int2 }
  | { format: "int4"; value: Secs2Int4 }
  | { format: "float8"; value: Secs2Float8 }
  | { format: "float4"; value: Secs2Float4 }
  | { format: "uint8"; value: Secs2UInt8 }
  | { format: "uint1"; value: Secs2UInt1 }
  | { format: "uint2"; value: Secs2UInt2 }
  | { format: "uint4"; value: Secs2UInt4 };

export type Secs2List = Secs2Item[];
export type Secs2Binary = number[];
export type Secs2Boolean = number[];
export type Secs2ASCII = string;
// 당장은 int8 / uint8 사이즈를 다룰 일이 없어 문제되지 않으나,
// 실제로 매우 큰 숫자가 오면 double 표현법을 넘어서므로 JSON 처리 로직을 JSON.parse에 의존할 수 없음
export type Secs2Int8 = number[];
export type Secs2Int1 = number[];
export type Secs2Int2 = number[];
export type Secs2Int4 = number[];
export type Secs2Float8 = number[];
export type Secs2Float4 = number[];
export type Secs2UInt8 = number[];
export type Secs2UInt1 = number[];
export type Secs2UInt2 = number[];
export type Secs2UInt4 = number[];

