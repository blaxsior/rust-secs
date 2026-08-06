"use client"

import { ContextMenuItem } from "@/components/ui/context-menu"
import type { Secs2NodeId } from "@/types/editor"
import type { Secs2SiblingInsertPosition } from "../store"
import { useSecs2EditorStore } from "../store"
import { createDefaultChildNode } from "./util/default-node"
import { runAfterContextMenuClose } from "./util/defer"

export function AddSiblingMenuItem({
  nodeId,
  position,
}: {
  nodeId: Secs2NodeId
  position: Secs2SiblingInsertPosition
}) {
  const createSibling = useSecs2EditorStore((state) => state.createSibling)

  return (
    <ContextMenuItem
      onClick={() => {
        runAfterContextMenuClose(() => {
          createSibling(nodeId, createDefaultChildNode(), position)
        })
      }}
    >
      {position === "above" ? "Add Above" : "Add Below"}
    </ContextMenuItem>
  )
}
