"use client"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { cn } from "@/lib/utils"
import { EditorInfoPanel } from "./EditorInfoPanel"
import { Secs2ItemNode } from "./Secs2ItemNode"
import { useSecs2EditorStore } from "./store"

type Secs2EditorProps = {
  className?: string
}

export default function Secs2Editor({ className }: Secs2EditorProps) {
  const rootId = useSecs2EditorStore((state) => state.rootId)
  const createRoot = useSecs2EditorStore((state) => state.createRoot)

  return (
    <div className={cn("grid gap-4 lg:grid-cols-[minmax(0,1.5fr)_minmax(320px,0.8fr)]", className)}>
      <Card className="min-h-[40rem]">
        <CardHeader>
          <CardTitle>SECS-II Items</CardTitle>
          <CardDescription>Click a node to inspect and edit it.</CardDescription>
        </CardHeader>
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
      </Card>

      <EditorInfoPanel />
    </div>
  )
}
