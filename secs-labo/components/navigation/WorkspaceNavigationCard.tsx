"use client";

import * as React from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ChevronDown, ChevronUp } from "lucide-react";
import { WorkspaceNavigation, type WorkspaceNavigationItem } from "./WorkspaceNavigation";

export function WorkspaceNavigationCard({
  description,
  items,
}: {
  description: string;
  items: readonly WorkspaceNavigationItem[];
}) {
  const [open, setOpen] = React.useState(false);

  return (
    <Card className="bg-white">
      <CardHeader>
        <div className="flex items-center justify-between gap-2">
          <div>
            <CardTitle className="text-sm tracking-[0.24em] text-slate-500">
              WORKSPACE Nav
            </CardTitle>
            <CardDescription>{description}</CardDescription>
          </div>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            onClick={() => setOpen((current) => !current)}
            aria-label={open ? "Close workspace menu" : "Open workspace menu"}
            aria-expanded={open}
          >
            {open ? <ChevronUp className="size-4" /> : <ChevronDown className="size-4" />}
          </Button>
        </div>
      </CardHeader>
      {open ? (
        <CardContent>
          <WorkspaceNavigation items={items} />
        </CardContent>
      ) : null}
    </Card>
  );
}
