import type { ReactNode } from "react";
import { WorkspaceNavigationCard } from "@/components/navigation/WorkspaceNavigationCard";
import WebLog from "@/components/weblog/WebLog";

const workspaceItems = [
  { href: "/protocols/secs1", label: "Overview", segment: null },
  {
    href: "/protocols/secs1/block_transfer",
    label: "Block Transfer",
    segment: "block_transfer",
  },
] as const;

export default function Secs1Layout({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-[calc(100vh-0px)] bg-[radial-gradient(circle_at_top_left,_rgba(34,197,94,0.16),_transparent_32%),radial-gradient(circle_at_top_right,_rgba(59,130,246,0.14),_transparent_28%),linear-gradient(180deg,_#fafafa_0%,_#f4f7fb_100%)] text-slate-900">
      <div className="flex min-h-screen w-full flex-col px-1.5 py-2 sm:px-2 lg:px-3">
        <header className="mb-3 flex flex-col gap-3 rounded-2xl border border-slate-200/80 bg-white/80 px-3 py-3 shadow-sm backdrop-blur sm:flex-row sm:items-center sm:justify-between">
          <div className="space-y-1">
            <p className="text-xs font-semibold uppercase tracking-[0.3em] text-emerald-600">
              Protocol Workbench
            </p>
            <h1 className="text-xl font-semibold tracking-tight text-slate-950">
              SECS-I Transport Lab
            </h1>
            <p className="text-sm text-slate-600">
              Explore serial transport settings, timing, and SECS message exchange planning.
            </p>
          </div>
        </header>

        <main className="grid min-h-0 flex-1 gap-2 lg:grid-cols-[280px_minmax(0,1fr)]">
          <aside className="flex min-h-0 flex-col gap-2">
            <WorkspaceNavigationCard
              description="Jump between SECS-I transport."
              items={workspaceItems}
            />
            <WebLog />
          </aside>

          <div className="min-w-0">{children}</div>
        </main>
      </div>
    </div>
  );
}
