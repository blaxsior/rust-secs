"use client";

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";

export type TransferOutputListItem = {
  id: number;
  value: string;
};

export function TransferOutputList({
  title,
  description,
  items,
  emptyMessage = "No output yet.",
}: {
  title: string;
  description: string;
  items: readonly TransferOutputListItem[];
  emptyMessage?: string;
}) {
  return (
    <Card className="min-h-0 bg-white">
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-44 rounded-xl border border-slate-200 bg-slate-50">
          {items.length > 0 ? (
            <ol className="divide-y divide-slate-200">
              {items.map((item) => (
                <li key={item.id} className="p-3">
                  <pre className="whitespace-pre-wrap break-words font-mono text-xs leading-5 text-slate-700">
                    {item.value}
                  </pre>
                </li>
              ))}
            </ol>
          ) : (
            <div className="p-3 text-sm text-slate-500">{emptyMessage}</div>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
