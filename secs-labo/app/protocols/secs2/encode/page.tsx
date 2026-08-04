'use client';
import HexEditor from "@/components/editor/hex/HexEditor";
import { useByteEditor } from "@/components/editor/hex/hooks/useEditor";
import { binRegex, hexRegex } from "@/components/editor/hex/util";
import { Button } from "@/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { binaryStrToNum, hexStrToNum, numToBinaryStr, numToHexStr } from "@/lib/convert";
import { ArrowRight } from "lucide-react";

export default function Secs2EncodePage() {
  const editorHandle = useByteEditor();

  const clearAll = () => {
    editorHandle.setBytes([]);
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
            <Button variant="outline" size="sm">
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
              <CardTitle>
                Message
              </CardTitle>
              <CardDescription>
                Build a SECS-II message before converting it into bytes.
              </CardDescription>
            </CardHeader>
            <CardContent className="px-0 pb-0">
              <div className="rounded-xl border border-dashed border-slate-300 bg-slate-50 p-4">
                <p className="text-sm font-medium text-slate-700">prepare message</p>
                <p className="mt-2 text-sm leading-6 text-slate-500">
                  Put the field editor here. This area can grow to fill the available space.
                </p>
              </div>
            </CardContent>
          </Card>

          <hr />

          <Card className="ring-0">
            <CardHeader className="px-0 pt-0">
              <CardTitle>
                Result
              </CardTitle>
              <CardDescription>
                Encoded byte output in binary and hex form.
              </CardDescription>
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
