"use client"

import * as React from "react"

import HexEditor from "@/components/editor/hex/HexEditor"
import { useByteEditor } from "@/components/editor/hex/hooks/useEditor"
import { binRegex, hexRegex } from "@/components/editor/hex/util"
import Secs2Editor from "@/components/editor/protocols/secs2/Secs2Editor"
import { Button } from "@/components/ui/button"
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { encode_secs2 } from "@/lib/wasm"
import { binaryStrToNum, hexStrToNum, numToBinaryStr, numToHexStr } from "@/lib/convert"
import { toSecs2Item } from "@/core/secs/editor-convert"
import type { EditorNode, EditorState } from "@/types/editor"
import { ArrowRight } from "lucide-react"

function createInitialSecs2EditorState(): EditorState {
  const rootId = crypto.randomUUID()

  const root: EditorNode = {
    id: rootId,
    parentId: null,
    format: "list",
    name: "Message",
    description: "Encode message root",
    value: {
      format: "list",
      children: [],
    },
  }

  return {
    rootId,
    nodes: new Map([[rootId, root]]),
  }
}

export default function Secs2EncodePage() {
  const [editorState, setEditorState] = React.useState<EditorState | null>(null)
  const editorHandle = useByteEditor();

  const clearAll = () => {
    editorHandle.setBytes([]);
  }

  const encodeMessage = () => {
    if (!editorState) {
      return
    }

    const item = toSecs2Item(editorState)
    const encoded = encode_secs2(JSON.stringify(item))
    editorHandle.setBytes(Array.from(encoded))
  }
  
  return (
    <div className="flex min-h-0 flex-col gap-4">
      <Card className="bg-white">
        <CardHeader>
          <div>
            <CardTitle>Encode</CardTitle>
            <CardDescription>Build a message and convert it into bytes.</CardDescription>
          </div>
          <CardAction className="space-x-2">
            <Button variant="outline" size="sm" onClick={encodeMessage}>
              encode <ArrowRight className="size-4" />
            </Button>
            <Button variant="destructive" size="sm" onClick={clearAll}>
              clear
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent className="flex flex-col gap-2">
          <Card className="ring-0">
            <CardHeader className="px-0 pt-0">
              <CardTitle>Message</CardTitle>
              <CardDescription>Build a SECS-II message before converting it into bytes.</CardDescription>
            </CardHeader>
            <CardContent className="px-0 pb-0">
              <Secs2Editor state={editorState} onChange={setEditorState} />
            </CardContent>
          </Card>

          <hr />

          <Card className="ring-0">
            <CardHeader className="px-0 pt-0">
              <CardTitle>Result</CardTitle>
              <CardDescription>Encoded byte output in binary and hex form.</CardDescription>
            </CardHeader>
            <CardContent className="px-0 pb-0">
              <div className="flex w-full gap-4 flex-col xl:flex-row">
                <HexEditor
                  readonly
                  name={"BINARY"}
                  validator={binRegex}
                  slotPerLine={4}
                  charPerSlot={8}
                  parseFunc={binaryStrToNum}
                  displayFunc={numToBinaryStr}
                  aria-label={"BIN_Viewer"}
                  {...editorHandle}
                  className="flex-2"
                />
                <HexEditor
                  readonly
                  name={"HEX"}
                  validator={hexRegex}
                  slotPerLine={4}
                  charPerSlot={2}
                  parseFunc={hexStrToNum}
                  displayFunc={numToHexStr}
                  aria-label={"HEX_Viewer"}
                  {...editorHandle}
                  className="flex-1"
                />
              </div>
            </CardContent>
          </Card>
        </CardContent>
      </Card>
    </div>
  );
}
