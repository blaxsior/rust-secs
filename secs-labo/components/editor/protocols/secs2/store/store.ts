"use client";

import { create } from "zustand";

import type {
  Secs2Node,
  Secs2NodeId,
  Secs2NodeInput,
  Secs2NodeState,
} from "@/types/editor";

type Secs2ListNode = Secs2Node & {
  value: {
    format: "list";
    children: Secs2NodeId[];
  };
};

export type Secs2SiblingInsertPosition = "above" | "below";

/**
 * secs2 editor 상태 값
 */
export type Secs2EditorState = {
  document: Secs2NodeState | null;
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
  /**
   * 특정 노드를 선택한다.
   * @param nodeId 선택할 노드. null이면 아무것도 선택 안함
   * @returns 
   */
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
  createSibling: (
    targetId: Secs2NodeId,
    childNode: Secs2NodeInput,
    position: Secs2SiblingInsertPosition,
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
  nodes: Record<Secs2NodeId, Secs2Node>,
  nodeId: Secs2NodeId,
  collected = new Set<Secs2NodeId>(),
) {
  const node = nodes[nodeId];

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
  nodes: Record<Secs2NodeId, Secs2Node>,
  openedNodeIds: Set<Secs2NodeId>,
  nodeId: Secs2NodeId,
) {
  let current = nodes[nodeId];

  while (current?.parentId) {
    const parentId = current.parentId;

    if (!openedNodeIds.has(parentId)) {
      return false;
    }

    current = nodes[parentId];
  }

  return Boolean(current);
}

/**
 * 자식 노드를 추가한다.
 * @param document editor node 상태
 * @param openedNodeIds 현재 열려 있는 노드 id의 집합
 * @param parent 부모 노드(항상 list)
 * @param childNode 추가할 자식 노드
 * @param insertIndex 자식 노드를 추가할 위치
 * @returns 
 */
function insertChildNode(
  document: Secs2NodeState,
  openedNodeIds: Set<Secs2NodeId>,
  parent: Secs2ListNode,
  childNode: Secs2NodeInput,
  insertIndex: number,
) {
  const childId = createEditorNodeId();
  const child: Secs2Node = {
    ...childNode,
    id: childId,
    parentId: parent.id,
  };
  const children = [...parent.value.children];
  children.splice(insertIndex, 0, childId);

  const nodes = { ...document.nodes };
  nodes[parent.id] = {
    ...parent,
    value: {
      ...parent.value,
      children,
    },
  };
  nodes[childId] = child;

  const nextOpenedNodeIds = new Set(openedNodeIds);
  nextOpenedNodeIds.add(parent.id);

  return {
    childId,
    document: {
      ...document,
      nodes,
    },
    openedNodeIds: nextOpenedNodeIds,
    selectedNodeId: childId,
  };
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
          nodes: {
            [root.id]: root,
          },
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
        const nodes = state.document?.nodes ?? {};

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
        const current = state.document?.nodes[node.id];

        if (!state.document || !current) {
          return state;
        }

        const nodes = { ...state.document.nodes };
        const deletedNodeIds = new Set<Secs2NodeId>();

        if (current.value.format === "list" && node.value.format !== "list") {
          for (const childId of current.value.children) {
            const childDeletedNodeIds = collectSubtreeNodeIds(nodes, childId);

            for (const deletedNodeId of childDeletedNodeIds) {
              deletedNodeIds.add(deletedNodeId);
              delete nodes[deletedNodeId];
            }
          }
        }

        nodes[node.id] = node;

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
        const parent = document?.nodes[parentId];

        if (!document || !parent || parent.value.format !== "list") {
          return state;
        }

        const nextState = insertChildNode(
          document,
          state.openedNodeIds,
          parent as Secs2ListNode,
          childNode,
          parent.value.children.length,
        );
        childId = nextState.childId;

        return {
          document: nextState.document,
          openedNodeIds: nextState.openedNodeIds,
          selectedNodeId: nextState.selectedNodeId,
        };
      });

      return childId;
    },

    createSibling: (targetId, childNode, position) => {
      let childId: Secs2NodeId | null = null;

      set((state) => {
        const document = state.document;
        const target = document?.nodes[targetId];

        if (!document || !target?.parentId) {
          return state;
        }

        const parent = document.nodes[target.parentId];

        if (!parent || parent.value.format !== "list") {
          return state;
        }

        const targetIndex = parent.value.children.indexOf(targetId);

        if (targetIndex < 0) {
          return state;
        }

        const insertIndex = position === "above" ? targetIndex : targetIndex + 1;
        const nextState = insertChildNode(
          document,
          state.openedNodeIds,
          parent as Secs2ListNode,
          childNode,
          insertIndex,
        );
        childId = nextState.childId;

        return {
          document: nextState.document,
          openedNodeIds: nextState.openedNodeIds,
          selectedNodeId: nextState.selectedNodeId,
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

        const node = state.document?.nodes[nodeId];

        if (!state.document || !node) {
          return state;
        }

        const deletedNodeIds = collectSubtreeNodeIds(
          state.document.nodes,
          nodeId,
        );
        const nodes = { ...state.document.nodes };

        if (node.parentId) {
          const parent = nodes[node.parentId];

          if (parent?.value.format === "list") {
            nodes[node.parentId] = {
              ...parent,
              value: {
                ...parent.value,
                children: parent.value.children.filter(
                  (childId) => childId !== nodeId,
                ),
              },
            };
          }
        }

        for (const deletedNodeId of deletedNodeIds) {
          delete nodes[deletedNodeId];
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
        nodes: structuredClone(document.nodes),
      };
    },

    putDocument: (document) => {
      set({
        document: document
          ? {
              rootId: document.rootId,
              nodes: structuredClone(document.nodes),
            }
          : null,
        selectedNodeId: null,
        openedNodeIds: document ? new Set([document.rootId]) : new Set(),
      });
    },
  }));
}
