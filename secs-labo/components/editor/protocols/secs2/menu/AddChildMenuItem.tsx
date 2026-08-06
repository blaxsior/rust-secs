"use client"

import { ContextMenuItem } from "@/components/ui/context-menu"
import type { Secs2NodeId } from "@/types/editor"
import { useSecs2EditorStore } from "../store"
import { createDefaultChildNode } from "./util/default-node"
import { runAfterContextMenuClose } from "./util/defer"

export function AddChildMenuItem({ nodeId }: { nodeId: Secs2NodeId }) {
  const createChild = useSecs2EditorStore((state) => state.createChild)

  return (
    <ContextMenuItem
      onClick={() => {
        runAfterContextMenuClose(() => {
          createChild(nodeId, createDefaultChildNode())
        })
      }}
    >
      Add Child
    </ContextMenuItem>
  )
}
