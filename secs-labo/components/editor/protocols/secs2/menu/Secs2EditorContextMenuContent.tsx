"use client"

import { ContextMenuContent } from "@/components/ui/context-menu"
import { useSecs2EditorStore } from "../store"
import { AddChildMenuItem } from "./AddChildMenuItem"
import { AddSiblingMenuItem } from "./AddSiblingMenuItem"
import { DeleteNodeMenuItem } from "./DeleteNodeMenuItem"

function useContextMenuItems() {
  const selectedNode = useSecs2EditorStore((state) =>
    state.selectedNodeId ? state.document?.nodes[state.selectedNodeId] ?? null : null
  )

  if (!selectedNode) {
    return null
  }

  return (
    <>
      {selectedNode.value.format === "list" ? (
        <AddChildMenuItem nodeId={selectedNode.id} />
      ) : null}
      {selectedNode.parentId ? (
        <AddSiblingMenuItem nodeId={selectedNode.id} position="above" />
      ) : null}
      {selectedNode.parentId ? (
        <AddSiblingMenuItem nodeId={selectedNode.id} position="below" />
      ) : null}
      <DeleteNodeMenuItem nodeId={selectedNode.id} />
    </>
  )
}

export function Secs2EditorContextMenuContent() {
  const content = useContextMenuItems()

  if (!content) {
    return null
  }

  return (
    <ContextMenuContent>
      {content}
    </ContextMenuContent>
  )
}
