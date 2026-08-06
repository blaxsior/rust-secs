"use client";

import Link from "next/link";
import { useSelectedLayoutSegment } from "next/navigation";
import { cn } from "@/lib/utils";

export type WorkspaceNavigationItem = {
  href: string;
  label: string;
  segment?: string | null;
};

export function WorkspaceNavigation({
  items,
}: {
  items: readonly WorkspaceNavigationItem[];
}) {
  const selectedSegment = useSelectedLayoutSegment();

  return (
    <div className="space-y-2">
      {items.map((item) => {
        const active = (item.segment ?? null) === selectedSegment;

        return (
          <Link
            key={item.href}
            href={item.href}
            aria-current={active ? "page" : undefined}
            className={cn(
              "block rounded-xl px-3 py-2 text-sm font-medium transition-colors",
              active
                ? "bg-slate-950 text-white"
                : "text-slate-700 hover:bg-slate-100"
            )}
          >
            {item.label}
          </Link>
        );
      })}
    </div>
  );
}
