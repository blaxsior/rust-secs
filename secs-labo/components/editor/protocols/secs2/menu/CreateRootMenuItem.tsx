"use client"

import { ContextMenuItem } from "@/components/ui/context-menu"
import { useSecs2EditorStore } from "../store"

export function CreateRootMenuItem() {
  const createRoot = useSecs2EditorStore((state) => state.createRoot)

  return (
    <ContextMenuItem onClick={createRoot}>
      Create Root
    </ContextMenuItem>
  )
}
