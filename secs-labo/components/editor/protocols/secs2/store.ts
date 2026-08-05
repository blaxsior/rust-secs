"use client"

import { create } from "zustand"

import type { EditorNode, EditorNodeId, EditorNodeInput, EditorState } from "@/types/editor"

type Secs2EditorStore = {
  rootId: EditorNodeId | null
  nodes: Map<EditorNodeId, EditorNode>
  selectedNodeId: EditorNodeId | null
  openedNodeIds: Set<EditorNodeId>
  createRoot: () => void
  deleteRoot: () => void
  selectNode: (nodeId: EditorNodeId | null) => void
  toggleNodeOpen: (nodeId: EditorNodeId) => void
  setNode: (node: EditorNode) => void
  createChild: (parentId: EditorNodeId, childNode: EditorNodeInput) => EditorNodeId | null
  deleteNode: (nodeId: EditorNodeId) => void
  getEditorState: () => EditorState | null
}

function createEditorNodeId(): EditorNodeId {
  return crypto.randomUUID()
}

function createRootNode(): EditorNode {
  const rootId = createEditorNodeId()

  return {
    id: rootId,
    parentId: null,
    format: "list",
    name: "Message",
    description: "Encode message root",
    value: {
      format: "list",
      children: [],
    },
  }
}

function collectSubtreeNodeIds(
  nodes: Map<EditorNodeId, EditorNode>,
  nodeId: EditorNodeId,
  collected = new Set<EditorNodeId>()
) {
  const node = nodes.get(nodeId)

  if (!node) {
    return collected
  }

  collected.add(nodeId)

  if (node.value.format === "list") {
    for (const childId of node.value.children) {
      collectSubtreeNodeIds(nodes, childId, collected)
    }
  }

  return collected
}

function isNodeVisible(
  nodes: Map<EditorNodeId, EditorNode>,
  openedNodeIds: Set<EditorNodeId>,
  nodeId: EditorNodeId
) {
  let current = nodes.get(nodeId)

  while (current?.parentId) {
    const parentId = current.parentId

    if (!openedNodeIds.has(parentId)) {
      return false
    }

    current = nodes.get(parentId)
  }

  return Boolean(current)
}

export const useSecs2EditorStore = create<Secs2EditorStore>((set, get) => ({
  rootId: null,
  nodes: new Map(),
  selectedNodeId: null,
  openedNodeIds: new Set(),

  createRoot: () => {
    const root = createRootNode()

    set({
      rootId: root.id,
      nodes: new Map([[root.id, root]]),
      selectedNodeId: null,
      openedNodeIds: new Set([root.id]),
    })
  },

  deleteRoot: () => {
    set({
      rootId: null,
      nodes: new Map(),
      selectedNodeId: null,
      openedNodeIds: new Set(),
    })
  },

  selectNode: (nodeId) => {
    set({ selectedNodeId: nodeId })
  },

  toggleNodeOpen: (nodeId) => {
    set((state) => {
      const openedNodeIds = new Set(state.openedNodeIds)

      if (openedNodeIds.has(nodeId)) {
        openedNodeIds.delete(nodeId)
      } else {
        openedNodeIds.add(nodeId)
      }

      return {
        openedNodeIds,
        selectedNodeId:
          state.selectedNodeId && isNodeVisible(state.nodes, openedNodeIds, state.selectedNodeId)
            ? state.selectedNodeId
            : null,
      }
    })
  },

  setNode: (node) => {
    set((state) => {
      if (!state.nodes.has(node.id)) {
        return state
      }

      const nodes = new Map(state.nodes)
      nodes.set(node.id, node)

      return { nodes }
    })
  },

  createChild: (parentId, childNode) => {
    let childId: EditorNodeId | null = null

    set((state) => {
      const parent = state.nodes.get(parentId)

      if (!parent || parent.value.format !== "list") {
        return state
      }

      childId = createEditorNodeId()
      const child: EditorNode = {
        ...childNode,
        id: childId,
        parentId,
      }
      const nodes = new Map(state.nodes)
      nodes.set(parentId, {
        ...parent,
        value: {
          ...parent.value,
          children: [...parent.value.children, childId],
        },
      })
      nodes.set(childId, child)

      const openedNodeIds = new Set(state.openedNodeIds)
      openedNodeIds.add(parentId)

      return {
        nodes,
        openedNodeIds,
        selectedNodeId: childId,
      }
    })

    return childId
  },

  deleteNode: (nodeId) => {
    set((state) => {
      if (nodeId === state.rootId) {
        return {
          rootId: null,
          nodes: new Map(),
          selectedNodeId: null,
          openedNodeIds: new Set(),
        }
      }

      const node = state.nodes.get(nodeId)

      if (!node) {
        return state
      }

      const deletedNodeIds = collectSubtreeNodeIds(state.nodes, nodeId)
      const nodes = new Map(state.nodes)

      if (node.parentId) {
        const parent = nodes.get(node.parentId)

        if (parent?.value.format === "list") {
          nodes.set(node.parentId, {
            ...parent,
            value: {
              ...parent.value,
              children: parent.value.children.filter((childId) => childId !== nodeId),
            },
          })
        }
      }

      for (const deletedNodeId of deletedNodeIds) {
        nodes.delete(deletedNodeId)
      }

      const openedNodeIds = new Set(state.openedNodeIds)
      for (const deletedNodeId of deletedNodeIds) {
        openedNodeIds.delete(deletedNodeId)
      }

      return {
        nodes,
        openedNodeIds,
        selectedNodeId:
          state.selectedNodeId && !deletedNodeIds.has(state.selectedNodeId)
            ? state.selectedNodeId
            : null,
      }
    })
  },

  getEditorState: () => {
    const { rootId, nodes } = get()

    if (!rootId) {
      return null
    }

    return structuredClone({
      rootId,
      nodes,
    })
  },
}))
