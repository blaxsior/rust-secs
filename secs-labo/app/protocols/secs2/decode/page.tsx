'use client';
import HexEditor from "@/components/editor/hex/HexEditor";
import { useByteEditor } from "@/components/editor/hex/hooks/useEditor";
import { binRegex, hexRegex } from "@/components/editor/hex/util";
import { Button } from "@/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { SecsItemToSMLSerializer } from "@/core/secs/sml/serializer";
import { binaryStrToNum, hexStrToNum, numToBinaryStr, numToHexStr } from "@/lib/convert";
import { decode_secs2 } from "@/lib/wasm";
import { Secs2Variant } from "@/types/secs2";
import { ArrowRight } from "lucide-react";
import { useState } from "react";

const smlSerializer = new SecsItemToSMLSerializer();

export default function Secs2DecodePage() {
  const editorHandle = useByteEditor();
  const [message, setMessage] = useState("");

  const decodeMessage = () => {
    const bytes = editorHandle.bytes;
    try {
      const result = decode_secs2(Uint8Array.from(bytes));
      const json: Secs2Variant = JSON.parse(result);
      const sml = smlSerializer.serialize(json);
      setMessage(sml);
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
        <CardHeader>
          <div>
            <CardTitle>Decode</CardTitle>
            <CardDescription>Read bytes and decode to SECS-II message.</CardDescription>
          </div>
          <CardAction className="space-x-2">
            <Button variant="outline" size="sm" onClick={decodeMessage}>
              decode <ArrowRight className="size-4" />
            </Button>
            <Button variant="destructive" size="sm" onClick={clearAll}>
              clear
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <Card className="ring-0">
            <CardHeader className="px-0 pt-0">
              <CardTitle>
                Input
              </CardTitle>
              <CardDescription>
                Enter the same byte stream in either binary or hex form.
              </CardDescription>
            </CardHeader>
            <CardContent className="px-0 pb-0">
              <div className="flex w-full gap-4 flex-col xl:flex-row">
                <HexEditor
                  name={"BINARY"}
                  validator={binRegex}
                  slotPerLine={4}
                  charPerSlot={8}
                  parseFunc={binaryStrToNum}
                  displayFunc={numToBinaryStr}
                  aria-label={"BIN_Editor"}
                  {...editorHandle}
                  className="flex-2"
                />

                <HexEditor
                  name={"HEX"}
                  validator={hexRegex}
                  slotPerLine={4}
                  charPerSlot={2}
                  parseFunc={hexStrToNum}
                  displayFunc={numToHexStr}
                  aria-label={"HEX_Editor"}
                  {...editorHandle}
                  className="flex-1"
                />
              </div>
            </CardContent>
          </Card>
          <hr/>
          <Card className="ring-0">
            <CardHeader className="px-0 pt-0">
              <CardTitle>
                Result
              </CardTitle>
              <CardDescription>
                Parsed SECS-II message structure or decode error details.
              </CardDescription>
            </CardHeader>
            <CardContent className="px-0 pb-0">
              <div className="rounded-2xl border border-slate-200 bg-slate-50/70 p-3 shadow-inner">
                <ScrollArea className="h-72 rounded-xl bg-white">
                  <pre className="min-h-72 whitespace-pre-wrap p-4 font-mono text-xs leading-5 text-slate-700">
                    {message || "No message yet."}
                  </pre>
                </ScrollArea>
              </div>
            </CardContent>
          </Card>
        </CardContent>
      </Card>
    </div>
  );
}
