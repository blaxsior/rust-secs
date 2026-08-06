"use client"

import { ContextMenuItem } from "@/components/ui/context-menu"
import type { Secs2NodeId } from "@/types/editor"
import { useSecs2EditorStore } from "../store"
import { runAfterContextMenuClose } from "./util/defer"

export function DeleteNodeMenuItem({ nodeId }: { nodeId: Secs2NodeId }) {
  const deleteNode = useSecs2EditorStore((state) => state.deleteNode)

  return (
    <ContextMenuItem
      variant="destructive"
      onClick={() => {
        runAfterContextMenuClose(() => {
          deleteNode(nodeId)
        })
      }}
    >
      Delete
    </ContextMenuItem>
  )
}
