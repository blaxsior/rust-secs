import type { EditorNode, EditorNodeId, EditorState, EditorValue } from "@/types/editor";
import type { Secs2Item } from "@/types/secs2";

function createEditorNodeId(): EditorNodeId {
  return crypto.randomUUID();
}

function buildEditorState(
  item: Secs2Item,
  parentId: EditorNodeId | null,
  nodes: Map<EditorNodeId, EditorNode>
): EditorNodeId {
  const id = createEditorNodeId();

  if (item.format === "list") {
    const children: EditorNodeId[] = [];

    for (const child of item.value) {
      const childId = buildEditorState(child, id, nodes);
      children.push(childId);
    }

    nodes.set(id, {
      id,
      parentId,
      format: item.format,
      value: {
        format: "list",
        children,
      },
    });
    return id;
  }

  nodes.set(id, {
    id,
    parentId,
    format: item.format,
    value: {
      format: item.format,
      value: item.value,
    } as EditorValue,
  });

  return id;
}

function buildSecs2Item(state: EditorState, nodeId: EditorNodeId): Secs2Item {
  const node = state.nodes.get(nodeId);

  if (!node) {
    throw new Error(`Editor node not found: ${nodeId}`);
  }

  if (node.value.format === "list") {
    return {
      format: "list",
      value: node.value.children.map((childId) => buildSecs2Item(state, childId)),
    };
  }

  return {
    format: node.value.format,
    value: node.value.value ?? [],
  } as Secs2Item;
}

export function fromSecs2Item(item: Secs2Item): EditorState {
  const nodes = new Map<EditorNodeId, EditorNode>();
  const rootId = buildEditorState(item, null, nodes);

  return {
    rootId,
    nodes,
  };
}

export function toSecs2Item(state: EditorState): Secs2Item {
  return buildSecs2Item(state, state.rootId);
}
