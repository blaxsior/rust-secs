import type { EditorNode, EditorNodeId, EditorNodeInput, EditorState } from "@/types/editor";

function createEditorNodeId(): EditorNodeId {
  return crypto.randomUUID();
}

export class EditorStore {
  private state: EditorState;

  constructor(root: EditorNode) {
    this.state = {
      rootId: root.id,
      nodes: new Map([[root.id, root]]),
    };
  }

  static fromState(state: EditorState): EditorStore {
    const store = Object.create(EditorStore.prototype) as EditorStore;
    store.state = {
      rootId: state.rootId,
      nodes: new Map(state.nodes),
    };
    return store;
  }

  getState(): EditorState {
    return structuredClone(this.state);
  }

  getNode(nodeId: EditorNodeId): EditorNode | undefined {
    return this.state.nodes.get(nodeId);
  }

  getChildren(nodeId: EditorNodeId): EditorNode[] {
    const node = this.state.nodes.get(nodeId);

    if (!node || node.value.format !== "list") {
      return [];
    }

    return node.value.children
      .map((childId) => this.state.nodes.get(childId))
      .filter((child): child is EditorNode => child !== undefined);
  }

  setNode(node: EditorNode): void {
    this.state.nodes.set(node.id, node);
  }

  updateNode(nodeId: EditorNodeId, patch: Partial<Omit<EditorNode, "id" | "parentId">>): void {
    const current = this.state.nodes.get(nodeId);

    if (!current) {
      throw new Error(`Editor node not found: ${nodeId}`);
    }

    this.state.nodes.set(nodeId, {
      ...current,
      ...patch,
    });
  }

  createNode(node: EditorNodeInput): EditorNodeId {
    const id = createEditorNodeId();

    this.state.nodes.set(id, {
      id,
      parentId: null,
      ...node,
    });

    return id;
  }

  deleteNode(nodeId: EditorNodeId): EditorNodeId {
    if (nodeId === this.state.rootId) {
      throw new Error("Cannot delete the root editor node.");
    }

    const target = this.state.nodes.get(nodeId);

    if (!target) {
      return nodeId;
    }

    if (target.value.format === "list") {
      for (const childId of [...target.value.children]) {
        this.deleteNode(childId);
      }
    }

    this.popNode(nodeId);
    this.state.nodes.delete(nodeId);

    return nodeId;
  }

  popNode(nodeId: EditorNodeId): EditorNodeId {
    if (nodeId === this.state.rootId) {
      throw new Error("Cannot pop the root editor node.");
    }

    const target = this.state.nodes.get(nodeId);

    if (!target) {
      return nodeId;
    }

    if (!target.parentId) {
      return nodeId;
    }

    const parent = this.state.nodes.get(target.parentId);

    if (parent?.value.format === "list") {
      this.state.nodes.set(target.parentId, {
        ...parent,
        value: {
          ...parent.value,
          children: parent.value.children.filter((childId) => childId !== nodeId),
        },
      });
    }

    this.state.nodes.set(nodeId, {
      ...target,
      parentId: null,
    });

    return nodeId;
  }

  pushNode(parentId: EditorNodeId, index: number, nodeId: EditorNodeId): EditorNodeId {
    const parent = this.state.nodes.get(parentId);

    if (!parent) {
      throw new Error(`Parent node not found: ${parentId}`);
    }

    if (parent.value.format !== "list") {
      throw new Error("Push is only allowed into list nodes.");
    }

    const child = this.state.nodes.get(nodeId);

    if (!child) {
      throw new Error(`Child node not found: ${nodeId}`);
    }

    if (nodeId === parentId) {
      throw new Error("Cannot push a node into itself.");
    }

    const nextChildren = [...parent.value.children];
    const nextIndex = Math.max(0, Math.min(index, nextChildren.length));

    nextChildren.splice(nextIndex, 0, nodeId);

    this.state.nodes.set(parentId, {
      ...parent,
      value: {
        ...parent.value,
        children: nextChildren,
      },
    });
    this.state.nodes.set(nodeId, {
      ...child,
      parentId,
    });

    return nodeId;
  }
}
