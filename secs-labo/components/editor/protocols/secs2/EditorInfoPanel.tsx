"use client"

import * as React from "react"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { SMLMapping } from "@/core/secs/const"
import type { Secs2Node, Secs2NodeId, Secs2NodeInput } from "@/types/editor"
import { useSecs2EditorStore } from "./store"

const FORMAT_OPTIONS = [
  "list",
  "binary",
  "boolean",
  "ascii",
  "int8",
  "int1",
  "int2",
  "int4",
  "float8",
  "float4",
  "uint8",
  "uint1",
  "uint2",
  "uint4",
] as const

function createEmptyValue(format: Secs2Node["format"]) {
  if (format === "list") {
    return {
      format,
      children: [],
    }
  }

  if (format === "ascii") {
    return {
      format,
      value: "",
    }
  }

  return {
    format,
    value: [],
  } as Secs2Node["value"]
}

function createDefaultChildNode(): Secs2NodeInput {
  return {
    format: "ascii",
    name: undefined,
    description: undefined,
    value: {
      format: "ascii",
      value: "",
    },
  }
}

function getDefaultScalarValue(format: Exclude<Secs2Node["format"], "list" | "ascii">) {
  return {
    format,
    value: [],
  } as Secs2Node["value"]
}

function getArrayValue(node: Secs2Node): number[] {
  if (node.value.format === "list") {
    return []
  }

  if (node.value.format === "ascii") {
    return []
  }

  return Array.isArray(node.value.value) ? node.value.value : []
}

export function EditorInfoPanel() {
  const node = useSecs2EditorStore((state) =>
    state.selectedNodeId ? state.document?.nodes.get(state.selectedNodeId) ?? null : null
  )

  if (!node) {
    return (
      <Card className="h-full">
        <CardHeader>
          <CardTitle>Editor Info</CardTitle>
          <CardDescription>Select a node to inspect or edit it.</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  return <EditorInfoPanelContent key={node.id} node={node} />
}

function EditorInfoPanelContent({ node }: { node: Secs2Node }) {
  const setNode = useSecs2EditorStore((state) => state.updateNode)
  const createChild = useSecs2EditorStore((state) => state.createChild)
  const deleteNode = useSecs2EditorStore((state) => state.deleteNode)
  const [draft, setDraft] = React.useState<Secs2Node>(node)

  const updateDraft = <K extends keyof Secs2Node>(key: K, value: Secs2Node[K]) => {
    setDraft((current) => {
      return {
        ...current,
        [key]: value,
      }
    })
  }

  const updateArrayValue = (nextValues: number[]) => {
    if (draft.value.format === "list" || draft.value.format === "ascii") {
      return
    }

    updateDraft("value", {
      format: draft.value.format,
      value: nextValues,
    } as Secs2Node["value"])
  }

  const appendArrayValue = () => {
    const nextValues = [...getArrayValue(draft), 0]
    updateArrayValue(nextValues)
  }

  const removeArrayValue = (index: number) => {
    const nextValues = getArrayValue(draft).filter((_, currentIndex) => currentIndex !== index)
    updateArrayValue(nextValues)
  }

  const removeChild = (childId: Secs2NodeId) => {
    if (draft.value.format === "list") {
      updateDraft("value", {
        ...draft.value,
        children: draft.value.children.filter((currentChildId) => currentChildId !== childId),
      })
    }

    deleteNode(childId)
  }

  return (
      <Card className="h-full">
        <CardHeader>
          <div className="flex items-start justify-between gap-2">
            <div>
              <CardTitle>Editor Info</CardTitle>
              <CardDescription>Inspect and edit the selected node.</CardDescription>
            </div>
            <Button
              type="button"
              variant="destructive"
              size="sm"
              onClick={() => deleteNode(node.id)}
            >
              Delete
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="editor-node-format">Format</Label>
          <select
            id="editor-node-format"
            value={draft.format}
            onChange={(event) => {
              const nextFormat = event.target.value as Secs2Node["format"]
              updateDraft("format", nextFormat)
              updateDraft(
                "value",
                nextFormat === "list"
                  ? createEmptyValue(nextFormat)
                  : nextFormat === "ascii"
                    ? createEmptyValue(nextFormat)
                    : getDefaultScalarValue(nextFormat)
              )
            }}
            className="h-8 w-full rounded-lg border border-input bg-background px-2 text-sm"
          >
            {FORMAT_OPTIONS.map((format) => (
              <option key={format} value={format}>
                {SMLMapping[format]}
              </option>
            ))}
          </select>
        </div>

        <div className="space-y-2">
          <Label htmlFor="editor-node-name">Name</Label>
          <Input
            id="editor-node-name"
            value={draft.name ?? ""}
            onChange={(event) => updateDraft("name", event.target.value)}
            placeholder="Optional node name"
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="editor-node-description">Description</Label>
          <textarea
            id="editor-node-description"
            value={draft.description ?? ""}
            onChange={(event) => updateDraft("description", event.target.value)}
            placeholder="Optional node description"
            className="min-h-24 w-full rounded-lg border border-input bg-background px-2 py-2 text-sm outline-none"
          />
        </div>

        {draft.value.format === "list" ? (
          <div className="space-y-2 rounded-xl border border-slate-200 bg-slate-50 p-3">
            <div className="flex items-center justify-between gap-2">
              <div className="text-sm font-medium">Children</div>
              <Button
                type="button"
                variant="outline"
                size="icon-sm"
                onClick={() => {
                  createChild(node.id, createDefaultChildNode())
                }}
                aria-label="Add child"
              >
                +
              </Button>
            </div>

            {draft.value.children.length === 0 ? (
              <div className="text-xs text-slate-500">No children yet.</div>
            ) : (
              <div className="space-y-2">
                {draft.value.children.map((childId) => (
                  <div
                    key={childId}
                    className="flex items-center justify-between gap-2 rounded-lg border border-slate-200 bg-white px-3 py-2 text-sm"
                  >
                    <div className="min-w-0">
                      <div className="truncate font-medium">{childId}</div>
                    </div>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon-sm"
                      onClick={() => removeChild(childId)}
                      aria-label="Remove child"
                    >
                      -
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </div>
        ) : (
          <div className="space-y-2 rounded-xl border border-slate-200 bg-slate-50 p-3">
            <div className="flex items-center justify-between gap-2">
              <div className="text-sm font-medium">Value</div>
              <Button
                type="button"
                variant="outline"
                size="icon-sm"
                onClick={appendArrayValue}
                aria-label="Add value"
              >
                +
              </Button>
            </div>

            {draft.value.format === "ascii" ? (
              <textarea
                value={draft.value.value}
                onChange={(event) =>
                  updateDraft("value", {
                    format: "ascii",
                    value: event.target.value,
                  })
                }
                className="min-h-24 w-full rounded-lg border border-input bg-background px-2 py-2 font-mono text-sm outline-none"
              />
            ) : (
              <div className="space-y-2">
                {getArrayValue(draft).length === 0 ? (
                  <div className="text-xs text-slate-500">No values yet.</div>
                ) : null}

                {getArrayValue(draft).map((value, index) => (
                  <div key={`${draft.id}-${index}`} className="flex items-center gap-2">
                    <Input
                      value={String(value)}
                      onChange={(event) => {
                        const nextValues = [...getArrayValue(draft)]
                        const nextValue = Number(event.target.value)
                        nextValues[index] = Number.isNaN(nextValue) ? 0 : nextValue
                        updateArrayValue(nextValues)
                      }}
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon-sm"
                      onClick={() => removeArrayValue(index)}
                      aria-label="Remove value"
                    >
                      -
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        <div className="flex gap-2">
          <Button type="button" onClick={() => setNode(draft)} className="flex-1">
            Save
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
