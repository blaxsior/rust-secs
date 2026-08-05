import type { Secs2Format, Secs2Item, Secs2Variant } from "./secs2";

export type EditorNodeId = string;

export type EditorNode = {
  id: EditorNodeId;
  parentId: EditorNodeId | null;
  format: Secs2Format;
  name?: string;
  description?: string;
  value: EditorValue;
};

type NonListSecs2Variant = Exclude<Secs2Variant, { format: "list" }>;

export type EditorValue =
  | { format: "list"; children: EditorNodeId[] }
  | NonListSecs2Variant;

export type EditorState = {
  rootId: EditorNodeId;
  nodes: Map<EditorNodeId, EditorNode>;
};

/**
 * Editor에 노드를 새롭게 추가할 때 사용
 */
export type EditorNodeInput = Omit<EditorNode, "id" | "parentId">;
