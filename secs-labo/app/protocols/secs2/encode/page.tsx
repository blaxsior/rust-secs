'use client';
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ArrowRight } from "lucide-react";

const fieldItems = [
  "Stream",
  "Function",
  "Need Reply",
  "Item",
];

export default function Secs2EncodePage() {

  return (
    <div className="flex min-h-0 flex-col gap-4">
      <Card className="bg-white">
        <CardHeader className="flex-row items-start justify-between gap-3">
          <div>
            <CardTitle>Encode</CardTitle>
            <CardDescription>Build a message and convert it into bytes.</CardDescription>
          </div>
          <Button variant="outline" size="sm">
            Export bytes <ArrowRight className="size-4" />
          </Button>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_320px]">
            <div className="rounded-xl border border-dashed border-slate-300 bg-slate-50 p-4">
              <p className="text-sm font-medium text-slate-700">Structured editor</p>
              <p className="mt-2 text-sm leading-6 text-slate-500">
                Put the field editor here. This area can grow to fill the available space.
              </p>
            </div>

            <Card className="bg-slate-950 text-slate-100">
              <CardHeader>
                <CardTitle className="text-base">Message fields</CardTitle>
                <CardDescription className="text-slate-300">
                  Field checklist and encoding hints.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <ScrollArea className="h-64 rounded-xl border border-slate-800">
                  <div className="space-y-2 p-3">
                    {fieldItems.map((item) => (
                      <div
                        key={item}
                        className="rounded-xl bg-slate-900/70 px-3 py-2 text-sm text-slate-200"
                      >
                        {item}
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              </CardContent>
            </Card>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
