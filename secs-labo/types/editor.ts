import type { Secs2Format, Secs2Variant } from "./secs2";

export type Secs2NodeId = string;

export type Secs2Node = {
  id: Secs2NodeId;
  parentId: Secs2NodeId | null;
  format: Secs2Format;
  name?: string;
  description?: string;
  value: Secs2NodeValue;
};

type NonListSecs2Variant = Exclude<Secs2Variant, { format: "list" }>;

export type Secs2NodeValue =
  | { format: "list"; children: Secs2NodeId[] }
  | NonListSecs2Variant;

export type Secs2NodeState = {
  rootId: Secs2NodeId;
  nodes: Record<Secs2NodeId, Secs2Node>;
};

/**
 * Editor에 노드를 새롭게 추가할 때 사용
 */
export type Secs2NodeInput = Omit<Secs2Node, "id" | "parentId">;
