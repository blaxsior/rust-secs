"use client"

import * as React from "react"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ContextMenu,
  ContextMenuTrigger,
} from "@/components/ui/context-menu"
import { cn } from "@/lib/utils"
import type { Secs2NodeState } from "@/types/editor"
import { EditorInfoPanel } from "./EditorInfoPanel"
import { Secs2EditorContextMenuContent } from "./menu/Secs2EditorContextMenuContent"
import { Secs2ItemNode } from "./Secs2ItemNode"
import { Secs2EditorProvider, useSecs2EditorStore } from "./store"

type Secs2EditorProps = {
  className?: string
}

export type Secs2EditorHandle = {
  getDocument: () => Secs2NodeState | null
  putDocument: (document: Secs2NodeState | null) => void
}

const Secs2Editor = React.forwardRef<Secs2EditorHandle, Secs2EditorProps>(
  function Secs2Editor(
    props,
    ref
  ) {
    return (
      <Secs2EditorProvider>
        <Secs2EditorInner {...props} ref={ref} />
      </Secs2EditorProvider>
    )
  })

const Secs2EditorInner = React.forwardRef<Secs2EditorHandle, Secs2EditorProps>(
  function Secs2EditorInner({ className }, ref) {
    const rootId = useSecs2EditorStore((state) => state.document?.rootId ?? null)
    const createRoot = useSecs2EditorStore((state) => state.createRoot)
    const getDocument = useSecs2EditorStore((state) => state.getDocument)
    const putDocument = useSecs2EditorStore((state) => state.putDocument)
    const selectNode = useSecs2EditorStore((state) => state.selectNode)

    React.useImperativeHandle(ref, () => ({
      getDocument,
      putDocument,
    }), [getDocument, putDocument])

    return (
      <div className={cn("grid gap-4 lg:grid-cols-[minmax(0,1.5fr)_minmax(320px,0.8fr)]", className)}>
        <Card className="min-h-[40rem]"
          onContextMenu={(e) => { e.preventDefault(); selectNode(null); }}
          // onClick={(e) => { e.preventDefault(); selectNode(null); }}
        >
          <CardHeader>
            <CardTitle>SECS-II Items</CardTitle>
            <CardDescription>Click a node to inspect and edit it.</CardDescription>
          </CardHeader>
          <ContextMenu>
            <ContextMenuTrigger
              onClick={() => console.log("click")}
              onContextMenu={() => console.log("contextmenu")}
              onTouchStart={() => console.log("touchstart")}
            >
              <CardContent className="space-y-2">
                {rootId ? (
                  <Secs2ItemNode nodeId={rootId} />
                ) : (
                  <div className="flex min-h-24 items-center justify-center rounded-xl border border-dashed border-slate-300 bg-slate-50 p-4">
                    <Button
                      type="button"
                      variant="outline"
                      onClick={createRoot}
                    >
                      +
                    </Button>
                  </div>
                )}
              </CardContent>
            </ContextMenuTrigger>
            <Secs2EditorContextMenuContent />
          </ContextMenu>
        </Card>

        <EditorInfoPanel />
      </div>
    )
  })

export default Secs2Editor
