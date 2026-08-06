"use client";

import { create } from "zustand";

import type {
  Secs2Node,
  Secs2NodeId,
  Secs2NodeInput,
  Secs2NodeState,
} from "@/types/editor";

export type Secs2EditorNodeMapState = {
  rootId: Secs2NodeId;
  nodes: Map<Secs2NodeId, Secs2Node>;
};

/**
 * secs2 editor 상태 값
 */
export type Secs2EditorState = {
  document: Secs2EditorNodeMapState | null;
  selectedNodeId: Secs2NodeId | null;
  openedNodeIds: Set<Secs2NodeId>;
};

/**
 * secs2 editor에서 수행 가능한 action 정의
 */
export type Secs2EditorAction = {
  /**
   * 루트 노드를 생성한다
   */
  createRoot: () => void;
  /**
   * 루트 노드를 제거한다.
   */
  deleteRoot: () => void;
  selectNode: (nodeId: Secs2NodeId | null) => void;
  toggleNodeOpen: (nodeId: Secs2NodeId) => void;
  /**
   * 노드 정보를 업데이트한다.
   * @param node 삽입할 노드
   */
  updateNode: (node: Secs2Node) => void;
  /**
   * list 노드에 자식을 추가한다
   * @param parentId 부모 노드 id
   * @param childNode 자식 노드 정보
   * @returns 추가된 자식 노드 id
   */
  createChild: (
    parentId: Secs2NodeId,
    childNode: Secs2NodeInput,
  ) => Secs2NodeId | null;
  /**
   * 특정 노드를 제거한다
   * @param nodeId node id
   * @returns
   */
  deleteNode: (nodeId: Secs2NodeId) => void;
  /**
   * 현재 노드 상태를 가져온다
   */
  getDocument: () => Secs2NodeState | null;
  /**
   * node 상태를 덮어쓴다
   * @param state
   * @returns
   */
  putDocument: (document: Secs2NodeState | null) => void;
};

export type Secs2EditorStore = Secs2EditorState & Secs2EditorAction;

/**
 * root id를 생성한다.
 * @returns
 */
function createEditorNodeId(): Secs2NodeId {
  return crypto.randomUUID();
}

/**
 * 루트 노드 empty secs list node 를 생성한다
 * @returns empty secs list
 */
function createRootNode(): Secs2Node {
  const rootId = createEditorNodeId();

  return {
    id: rootId,
    parentId: null,
    format: "list",
    value: {
      format: "list",
      children: [],
    },
  };
}

/**
 * 현재 노드의 모든 하위 노드 id를 찾는다 (삭제 목적)
 * @param nodes 전체 노드 목록
 * @param nodeId 현재 노드 ID 값
 * @param collected 수집된 노드 ID 집합
 */
function collectSubtreeNodeIds(
  nodes: Map<Secs2NodeId, Secs2Node>,
  nodeId: Secs2NodeId,
  collected = new Set<Secs2NodeId>(),
) {
  const node = nodes.get(nodeId);

  if (!node) {
    return collected;
  }

  collected.add(nodeId);

  if (node.value.format === "list") {
    for (const childId of node.value.children) {
      collectSubtreeNodeIds(nodes, childId, collected);
    }
  }

  return collected;
}

/**
 * 대상 노드가 현재 열려있는 상태인지 확인한다.
 * @param nodes
 * @param openedNodeIds
 * @param nodeId
 * @returns
 */
function isNodeVisible(
  nodes: Map<Secs2NodeId, Secs2Node>,
  openedNodeIds: Set<Secs2NodeId>,
  nodeId: Secs2NodeId,
) {
  let current = nodes.get(nodeId);

  while (current?.parentId) {
    const parentId = current.parentId;

    if (!openedNodeIds.has(parentId)) {
      return false;
    }

    current = nodes.get(parentId);
  }

  return Boolean(current);
}

/**
 * secs2 editor store 생성
 * @param initState 초기 상태 값
 * @returns
 */
export function createSecs2EditorStore(initState?: Secs2EditorState) {
  return create<Secs2EditorStore>((set, get) => ({
    document: initState?.document ?? null,
    selectedNodeId: initState?.selectedNodeId ?? null,
    openedNodeIds: initState?.openedNodeIds ?? new Set(),

    createRoot: () => {
      const root = createRootNode();

      set({
        document: {
          rootId: root.id,
          nodes: new Map([[root.id, root]]),
        },
        selectedNodeId: null,
        openedNodeIds: new Set([root.id]),
      });
    },

    deleteRoot: () => {
      set({
        document: null,
        selectedNodeId: null,
        openedNodeIds: new Set(),
      });
    },

    selectNode: (nodeId) => {
      set({ selectedNodeId: nodeId });
    },

    toggleNodeOpen: (nodeId) => {
      set((state) => {
        const openedNodeIds = new Set(state.openedNodeIds);
        const nodes = state.document?.nodes ?? new Map();

        if (openedNodeIds.has(nodeId)) {
          openedNodeIds.delete(nodeId);
        } else {
          openedNodeIds.add(nodeId);
        }

        return {
          openedNodeIds,
          selectedNodeId:
            state.selectedNodeId &&
            isNodeVisible(nodes, openedNodeIds, state.selectedNodeId)
              ? state.selectedNodeId
              : null,
        };
      });
    },

    updateNode: (node) => {
      set((state) => {
        const current = state.document?.nodes.get(node.id);

        if (!state.document || !current) {
          return state;
        }

        const nodes = new Map(state.document.nodes);
        const deletedNodeIds = new Set<Secs2NodeId>();

        if (current.value.format === "list" && node.value.format !== "list") {
          for (const childId of current.value.children) {
            const childDeletedNodeIds = collectSubtreeNodeIds(nodes, childId);

            for (const deletedNodeId of childDeletedNodeIds) {
              deletedNodeIds.add(deletedNodeId);
              nodes.delete(deletedNodeId);
            }
          }
        }

        nodes.set(node.id, node);

        const openedNodeIds = new Set(state.openedNodeIds);
        for (const deletedNodeId of deletedNodeIds) {
          openedNodeIds.delete(deletedNodeId);
        }

        return {
          document: {
            ...state.document,
            nodes,
          },
          openedNodeIds,
          selectedNodeId:
            state.selectedNodeId && !deletedNodeIds.has(state.selectedNodeId)
              ? state.selectedNodeId
              : node.id,
        };
      });
    },

    createChild: (parentId, childNode) => {
      let childId: Secs2NodeId | null = null;

      set((state) => {
        const document = state.document;
        const parent = document?.nodes.get(parentId);

        if (!document || !parent || parent.value.format !== "list") {
          return state;
        }

        childId = createEditorNodeId();
        const child: Secs2Node = {
          ...childNode,
          id: childId,
          parentId,
        };
        const nodes = new Map(document.nodes);
        nodes.set(parentId, {
          ...parent,
          value: {
            ...parent.value,
            children: [...parent.value.children, childId],
          },
        });
        nodes.set(childId, child);

        const openedNodeIds = new Set(state.openedNodeIds);
        openedNodeIds.add(parentId);

        return {
          document: {
            ...document,
            nodes,
          },
          openedNodeIds,
          selectedNodeId: childId,
        };
      });

      return childId;
    },

    deleteNode: (nodeId) => {
      set((state) => {
        if (nodeId === state.document?.rootId) {
          return {
            document: null,
            selectedNodeId: null,
            openedNodeIds: new Set(),
          };
        }

        const node = state.document?.nodes.get(nodeId);

        if (!state.document || !node) {
          return state;
        }

        const deletedNodeIds = collectSubtreeNodeIds(
          state.document.nodes,
          nodeId,
        );
        const nodes = new Map(state.document.nodes);

        if (node.parentId) {
          const parent = nodes.get(node.parentId);

          if (parent?.value.format === "list") {
            nodes.set(node.parentId, {
              ...parent,
              value: {
                ...parent.value,
                children: parent.value.children.filter(
                  (childId) => childId !== nodeId,
                ),
              },
            });
          }
        }

        for (const deletedNodeId of deletedNodeIds) {
          nodes.delete(deletedNodeId);
        }

        const openedNodeIds = new Set(state.openedNodeIds);
        for (const deletedNodeId of deletedNodeIds) {
          openedNodeIds.delete(deletedNodeId);
        }

        return {
          document: {
            ...state.document,
            nodes,
          },
          openedNodeIds,
          selectedNodeId:
            state.selectedNodeId && !deletedNodeIds.has(state.selectedNodeId)
              ? state.selectedNodeId
              : null,
        };
      });
    },

    getDocument: () => {
      const { document } = get();

      if (!document) {
        return null;
      }

      return {
        rootId: document.rootId,
        nodes: structuredClone(Object.fromEntries(document.nodes)),
      };
    },

    putDocument: (document) => {
      set({
        document: document
          ? {
              rootId: document.rootId,
              nodes: new Map(Object.entries(structuredClone(document.nodes))),
            }
          : null,
        selectedNodeId: null,
        openedNodeIds: document ? new Set([document.rootId]) : new Set(),
      });
    },
  }));
}
