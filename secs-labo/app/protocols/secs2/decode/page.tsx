'use client';
import HexEditor from "@/components/editor/hex/HexEditor";
import { useByteEditor } from "@/components/editor/hex/hooks/useEditor";
import { binRegex, hexRegex } from "@/components/editor/hex/util";
import { Button } from "@/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { binaryStrToNum, hexStrToNum, numToBinaryStr, numToHexStr } from "@/lib/convert";
import { decode_secs2 } from "@/lib/wasm";
import { ArrowRight, DeleteIcon } from "lucide-react";
import { useState } from "react";

export default function Secs2DecodePage() {
  const editorHandle = useByteEditor();
  const [message, setMessage] = useState("");

  const decodeMessage = () => {
    let bytes = editorHandle.bytes;
    try {
      let result = decode_secs2(Uint8Array.from(bytes));
      setMessage(result);
    } catch (e) {
      console.log("error occured", e);
      const message = e instanceof Error ? e.message : String(e);
      setMessage(message);
    }
  };

  const clearAll = () => {
    setMessage("");
    editorHandle.setBytes([]);
  }

  return (
    <div className="flex min-h-0 flex-col gap-4">
      <Card className="bg-white">
        <CardHeader className="flex-row items-start justify-between gap-3">
          <div>
            <CardTitle>Decode</CardTitle>
            <CardDescription>Read bytes and decode to SECS-II message.</CardDescription>
          </div>
          <CardAction>
            <Button variant="secondary" size="sm" onClick={decodeMessage}>
              decode <ArrowRight className="size-4" />
            </Button>
            <Button variant="destructive" size="sm" onClick={clearAll}>
              clear
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_320px]">
            <HexEditor
              name={"BINARY"}
              validator={binRegex}
              itemPerLine={4}
              charPerItem={8}
              parseFunc={binaryStrToNum}
              displayFunc={numToBinaryStr}
              aria-label={"BIN_Editor"}
              {...editorHandle}
            />
            <HexEditor
              name={"HEX"}
              validator={hexRegex}
              itemPerLine={4}
              charPerItem={2}
              parseFunc={hexStrToNum}
              displayFunc={numToHexStr}
              aria-label={"HEX_Editor"}
              {...editorHandle}
            />

            <Card className="bg-white xl:col-span-2">
              <CardHeader>
                <CardTitle className="text-base">Decoded Message</CardTitle>
                <CardDescription>
                  Parsed SECS-II message output.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <ScrollArea className="h-64 rounded-xl border border-slate-200">
                  <pre className="min-h-64 whitespace-pre-wrap p-3 font-mono text-xs leading-5 text-slate-700">
                    {message || "No message yet."}
                  </pre>
                </ScrollArea>
              </CardContent>
            </Card>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
