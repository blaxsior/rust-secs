"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";

const items = [
  { href: "/protocols/secs2", label: "Overview" },
  { href: "/protocols/secs2/encode", label: "Encode" },
  { href: "/protocols/secs2/decode", label: "Decode" },
] as const;

export function Secs2WorkspaceNav() {
  const pathname = usePathname();

  return (
    <div className="space-y-2">
      {items.map((item) => {
        const active = pathname === item.href;

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
