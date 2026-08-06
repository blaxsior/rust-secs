"use client"

import { ContextMenuItem } from "@/components/ui/context-menu"
import { useSecs2EditorStore } from "../store"
import { runAfterContextMenuClose } from "./util/defer"

export function CreateRootMenuItem() {
  const createRoot = useSecs2EditorStore((state) => state.createRoot)

  return (
    <ContextMenuItem
      onClick={() => {
        runAfterContextMenuClose(createRoot)
      }}
    >
      Create Root
    </ContextMenuItem>
  )
}
