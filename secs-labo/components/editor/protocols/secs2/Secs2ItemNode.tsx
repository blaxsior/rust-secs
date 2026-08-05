"use client"

import { SMLMapping } from "@/core/secs/const"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import type { EditorNode, EditorNodeId } from "@/types/editor"
import { cn } from "@/lib/utils"
import { ChevronDown, ChevronRight } from "lucide-react"
import { useSecs2EditorStore } from "./store"

/**
 * 값을 문자열 형태로 표현
 * @param node 대상 노드
 * @returns 표현된 값
 */
function stringifyValue(node: EditorNode) {
  if (node.value.format === "list") {
    return `${node.value.children.length} children`
  }

  if (node.value.format === "ascii") {
    return node.value.value ?? ""
  }

  return (node.value.value ?? []).join(", ")
}

/**
 * 아이템 길이를 표현
 * @param node 대상 노드
 * @returns 표현된 값
 */
function stringifyLength(node: EditorNode): string {
  if (node.value.format === "list") {
    const length = node.value.children.length;
    return length > 0 ? `[${length.toString()}]` : "";
  }

  if (node.value.format === "ascii") {
    return "";
  }

  const length = node.value.value.length;
  return length > 1 ? `[${length.toString()}]` : "";
}



export function Secs2ItemNode({
  nodeId,
}: {
  nodeId: EditorNodeId
}) {
  const node = useSecs2EditorStore((state) => state.nodes.get(nodeId))
  const isSelected = useSecs2EditorStore((state) => state.selectedNodeId === nodeId)
  const expanded = useSecs2EditorStore((state) => state.openedNodeIds.has(nodeId))
  const selectNode = useSecs2EditorStore((state) => state.selectNode)
  const toggleNodeOpen = useSecs2EditorStore((state) => state.toggleNodeOpen)

  if (!node) {
    return null
  }

  const children = node.value.format === "list" ? node.value.children : []
  const canExpand = node.value.format === "list" && children.length > 0

  return (
    <div className="space-y-2">
      <Card
        className={cn(
          "relative border px-0 py-0 shadow-none",
          isSelected
            ? "border-sky-500 bg-sky-50"
            : "border-slate-200 bg-white/80"
        )}
      >
        <CardContent className="space-y-2 px-3 py-2">
          <button
            type="button"
            onClick={(event) => {
              event.stopPropagation()
              selectNode(nodeId)
            }}
            className="block w-full text-left"
          >
            <div className="flex items-center justify-between gap-2">
              <div className="font-mono text-xs uppercase tracking-wide text-slate-500">
                {`${SMLMapping[node.format]}${stringifyLength(node)}`}
              </div>
              <div className="text-xs text-slate-500">{node.id}</div>
            </div>
            <div className="mt-1 text-sm font-medium">
              {node.name?.trim() ? node.name : "Untitled node"}
            </div>
            {node.description?.trim() ? (
              <div className="mt-1 line-clamp-2 text-xs text-slate-500">
                {node.description}
              </div>
            ) : null}
            <div className="mt-2 font-mono text-xs text-slate-500">
              {stringifyValue(node)}
            </div>
          </button>

          {canExpand ? (
            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              onClick={(event) => {
                event.stopPropagation()
                toggleNodeOpen(nodeId)
              }}
              aria-label={expanded ? "Collapse node" : "Expand node"}
              className="absolute right-2 bottom-2 rounded-lg"
            >
              {expanded ? <ChevronDown /> : <ChevronRight />}
            </Button>
          ) : null}
        </CardContent>
      </Card>

      {expanded && children.length > 0 ? (
        <div className="ml-4 border-l border-slate-200 pl-4">
          {children.map((childId) => (
            <Secs2ItemNode
              key={childId}
              nodeId={childId}
            />
          ))}
        </div>
      ) : null}
    </div>
  )
}
