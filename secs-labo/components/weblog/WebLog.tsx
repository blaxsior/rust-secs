"use client";

import { useWebLogStore } from "@/stores/weblog/useWebLogStore";
import {
    Card,
    CardAction,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { useState } from "react";
import { ScrollArea } from "../ui/scroll-area";

const logLevelClassName: Record<string, string> = {
    ERROR: "text-red-600",
    WARN: "text-amber-600",
    INFO: "text-sky-600",
    DEBUG: "text-slate-500",
    TRACE: "text-zinc-400",
};

/**
 * wasm에서 전달된 로그를 출력하는 영역
 */
function WebLog() {
    const [limitInput, setLimitInput] = useState(100);

    const log = useWebLogStore((store) => store.logs);
    const limit = useWebLogStore((store) => store.limit);

    const clearLog = useWebLogStore((store) => store.clear);
    const setLimit = useWebLogStore((store) => store.setLineLimit);

    const setLineCount = () => {
        if (Number.isNaN(limitInput) || limitInput < 0) return;
        setLimit(limitInput);
    }

    const visibleLogs = log.slice(-limit);

    return (
        <Card size="sm" className="flex h-64 min-h-40 resize-y flex-col overflow-auto">
            <CardHeader>
                <CardTitle>Web Log</CardTitle>
                <CardDescription>
                    {visibleLogs.length} / {limit} entries
                </CardDescription>
                <CardAction className={cn("flex flex-row")}>
                    <div className="flex items-center gap-2">
                        <Input
                            type="number"
                            min={0}
                            value={Number.isNaN(limitInput) ? "" : limitInput}
                            onChange={(event) => {
                                const value = event.target.value;
                                setLimitInput(value === "" ? Number.NaN : Number(value));
                            }}
                            className="w-20"
                        />
                        <Button type="button" variant="outline" onClick={setLineCount}>
                            set
                        </Button>
                        <Button type="button" variant="ghost" onClick={clearLog}>
                            clear
                        </Button>
                    </div>
                </CardAction>
            </CardHeader>
            <CardContent className="min-h-0 flex-1">
                <ScrollArea className="h-full">
                    {visibleLogs.length === 0 ? (
                        <div className="text-muted-foreground">No logs yet.</div>
                    ) : (
                        visibleLogs.map((item, index) => (
                            <div
                                key={`${item.level}-${item.msg}-${index}`}
                                className="rounded-md border border-border/60 bg-muted/40 px-2 py-1"
                            >
                                <span className={cn("mr-2 font-semibold uppercase text-muted-foreground", logLevelClassName[item.level] ?? "bg-muted text-muted-foreground ring-border")}>
                                    {item.level}
                                </span>
                                <span>{item.msg}</span>
                            </div>
                        ))
                    )}
                </ScrollArea>
            </CardContent>
            <CardFooter className="justify-between text-xs text-muted-foreground">
                <span>Store limit: {limit}</span>
            </CardFooter>
        </Card>
    );
}

export default WebLog;
