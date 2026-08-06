import type { Secs2Node, Secs2NodeId, Secs2NodeInput, Secs2NodeState } from "@/types/editor";

function createEditorNodeId(): Secs2NodeId {
  return crypto.randomUUID();
}

export class EditorStore {
  private state: Secs2NodeState;

  constructor(root: Secs2Node) {
    this.state = {
      rootId: root.id,
      nodes: {
        [root.id]: root,
      },
    };
  }

  static fromState(state: Secs2NodeState): EditorStore {
    const store = Object.create(EditorStore.prototype) as EditorStore;
    store.state = structuredClone(state);
    return store;
  }

  getState(): Secs2NodeState {
    return structuredClone(this.state);
  }

  getNode(nodeId: Secs2NodeId): Secs2Node | undefined {
    return this.state.nodes[nodeId];
  }

  getChildren(nodeId: Secs2NodeId): Secs2Node[] {
    const node = this.state.nodes[nodeId];

    if (!node || node.value.format !== "list") {
      return [];
    }

    return node.value.children
      .map((childId) => this.state.nodes[childId])
      .filter((child): child is Secs2Node => child !== undefined);
  }

  setNode(node: Secs2Node): void {
    this.state.nodes[node.id] = node;
  }

  updateNode(nodeId: Secs2NodeId, patch: Partial<Omit<Secs2Node, "id" | "parentId">>): void {
    const current = this.state.nodes[nodeId];

    if (!current) {
      throw new Error(`Editor node not found: ${nodeId}`);
    }

    this.state.nodes[nodeId] = {
      ...current,
      ...patch,
    };
  }

  createNode(node: Secs2NodeInput): Secs2NodeId {
    const id = createEditorNodeId();

    this.state.nodes[id] = {
      id,
      parentId: null,
      ...node,
    };

    return id;
  }

  deleteNode(nodeId: Secs2NodeId): Secs2NodeId {
    if (nodeId === this.state.rootId) {
      throw new Error("Cannot delete the root editor node.");
    }

    const target = this.state.nodes[nodeId];

    if (!target) {
      return nodeId;
    }

    if (target.value.format === "list") {
      for (const childId of [...target.value.children]) {
        this.deleteNode(childId);
      }
    }

    this.popNode(nodeId);
    delete this.state.nodes[nodeId];

    return nodeId;
  }

  popNode(nodeId: Secs2NodeId): Secs2NodeId {
    if (nodeId === this.state.rootId) {
      throw new Error("Cannot pop the root editor node.");
    }

    const target = this.state.nodes[nodeId];

    if (!target) {
      return nodeId;
    }

    if (!target.parentId) {
      return nodeId;
    }

    const parent = this.state.nodes[target.parentId];

    if (parent?.value.format === "list") {
      this.state.nodes[target.parentId] = {
        ...parent,
        value: {
          ...parent.value,
          children: parent.value.children.filter((childId) => childId !== nodeId),
        },
      };
    }

    this.state.nodes[nodeId] = {
      ...target,
      parentId: null,
    };

    return nodeId;
  }

  pushNode(parentId: Secs2NodeId, index: number, nodeId: Secs2NodeId): Secs2NodeId {
    const parent = this.state.nodes[parentId];

    if (!parent) {
      throw new Error(`Parent node not found: ${parentId}`);
    }

    if (parent.value.format !== "list") {
      throw new Error("Push is only allowed into list nodes.");
    }

    const child = this.state.nodes[nodeId];

    if (!child) {
      throw new Error(`Child node not found: ${nodeId}`);
    }

    if (nodeId === parentId) {
      throw new Error("Cannot push a node into itself.");
    }

    const nextChildren = [...parent.value.children];
    const nextIndex = Math.max(0, Math.min(index, nextChildren.length));

    nextChildren.splice(nextIndex, 0, nodeId);

    this.state.nodes[parentId] = {
      ...parent,
      value: {
        ...parent.value,
        children: nextChildren,
      },
    };
    this.state.nodes[nodeId] = {
      ...child,
      parentId,
    };

    return nodeId;
  }
}
