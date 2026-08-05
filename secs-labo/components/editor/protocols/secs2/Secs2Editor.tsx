"use client"

import * as React from "react"

import { EditorStore } from "@/core/secs/editor"
import type { EditorNode, EditorNodeId, EditorState } from "@/types/editor"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { cn } from "@/lib/utils"
import { EditorInfoPanel } from "./EditorInfoPanel"
import { Secs2ItemNode } from "./Secs2ItemNode"

type Secs2EditorProps = {
  state: EditorState | null
  onChange?: (nextState: EditorState | null) => void
  className?: string
}

function fromListChildrenLength(store: EditorStore, nodeId: EditorNodeId) {
  const node = store.getNode(nodeId)

  if (!node || node.value.format !== "list") {
    return 0
  }

  return node.value.children.length
}

function createInitialSecs2EditorState(): EditorState {
  const rootId = crypto.randomUUID()

  const root: EditorNode = {
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

  return {
    rootId,
    nodes: new Map([[rootId, root]]),
  }
}

export default function Secs2Editor({ state, onChange, className }: Secs2EditorProps) {
  const storeRef = React.useRef<EditorStore | null>(state ? EditorStore.fromState(state) : null)
  const previousRootIdRef = React.useRef<EditorNodeId | null>(state?.rootId ?? null)
  const [selectedNodeId, setSelectedNodeId] = React.useState<EditorNodeId | null>(null)
  const [expandedNodeIds, setExpandedNodeIds] = React.useState<Set<EditorNodeId>>(new Set())

  React.useEffect(() => {
    storeRef.current = state ? EditorStore.fromState(state) : null
  }, [state])

  React.useEffect(() => {
    const nextRootId = state?.rootId ?? null
    const previousRootId = previousRootIdRef.current

    if (previousRootId === nextRootId) {
      return
    }

    previousRootIdRef.current = nextRootId
    setSelectedNodeId(null)
    setExpandedNodeIds(nextRootId ? new Set([nextRootId]) : new Set())
  }, [state?.rootId])

  const store = storeRef.current
  const rootId = store?.getState().rootId ?? null
  const selectedNode = selectedNodeId && store ? store.getNode(selectedNodeId) ?? null : null

  const notifyChange = () => {
    onChange?.(store?.getState() ?? null)
  }

  const handleSaveNode = (nextNode: EditorNode) => {
    if (!store) {
      return
    }

    store.setNode(nextNode)
    notifyChange()
  }

  const handleCreateChild = (parentId: EditorNodeId, childNode: Omit<EditorNode, "id" | "parentId">) => {
    if (!store) {
      return
    }

    const childId = store.createNode(childNode)
    store.pushNode(parentId, fromListChildrenLength(store, parentId), childId)
    setExpandedNodeIds((current) => new Set(current).add(parentId))
    setSelectedNodeId(childId)
    notifyChange()
  }

  const handleDeleteNode = (nodeId: EditorNodeId) => {
    if (!store) {
      return
    }

    if (nodeId === rootId) {
      onChange?.(null)
      return
    }

    store.deleteNode(nodeId)

    if (!selectedNodeId || !store.getNode(selectedNodeId)) {
      setSelectedNodeId(null)
    }

    notifyChange()
  }

  const handleCreateRoot = () => {
    const createdState = createInitialSecs2EditorState()
    storeRef.current = EditorStore.fromState(createdState)
    previousRootIdRef.current = createdState.rootId
    setExpandedNodeIds(new Set([createdState.rootId]))
    setSelectedNodeId(null)
    onChange?.(createdState)
  }

  const handleToggleExpandNode = (nodeId: EditorNodeId) => {
    setExpandedNodeIds((current) => {
      const next = new Set(current)

      if (next.has(nodeId)) {
        next.delete(nodeId)
      } else {
        next.add(nodeId)
      }

      return next
    })
  }

  return (
    <div className={cn("grid gap-4 lg:grid-cols-[minmax(0,1.5fr)_minmax(320px,0.8fr)]", className)}>
      <Card className="min-h-[40rem]">
        <CardHeader>
          <CardTitle>SECS-II Items</CardTitle>
          <CardDescription>Click a node to inspect and edit it.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          {store && rootId ? (
            <Secs2ItemNode
              store={store}
              nodeId={rootId}
              selectedNodeId={selectedNodeId}
              onSelectNode={(nodeId) => setSelectedNodeId(nodeId)}
              expandedNodeIds={expandedNodeIds}
              onToggleExpandNode={handleToggleExpandNode}
            />
          ) : (
            <div className="flex min-h-24 items-center justify-center rounded-xl border border-dashed border-slate-300 bg-slate-50 p-4">
              <Button
                type="button"
                variant="outline"
                onClick={handleCreateRoot}
              >
                +
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      <EditorInfoPanel
        node={selectedNode}
        onSave={handleSaveNode}
        onCreateChild={handleCreateChild}
        onDeleteNode={handleDeleteNode}
      />
    </div>
  )
}
