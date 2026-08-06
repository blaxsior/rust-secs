import type { ReactNode } from "react";
import { WorkspaceNavigationCard } from "@/components/navigation/WorkspaceNavigationCard";
import WebLog from "@/components/weblog/WebLog";

const workspaceItems = [
  { href: "/protocols/hsms", label: "Overview", segment: null },
] as const;

export default function HsmsLayout({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-[calc(100vh-0px)] bg-[radial-gradient(circle_at_top_left,_rgba(34,197,94,0.16),_transparent_32%),radial-gradient(circle_at_top_right,_rgba(59,130,246,0.14),_transparent_28%),linear-gradient(180deg,_#fafafa_0%,_#f4f7fb_100%)] text-slate-900">
      <div className="mx-auto flex min-h-screen w-full max-w-7xl flex-col px-4 py-4 sm:px-6 lg:px-8">
        <header className="mb-4 flex flex-col gap-3 rounded-2xl border border-slate-200/80 bg-white/80 px-4 py-4 shadow-sm backdrop-blur sm:flex-row sm:items-center sm:justify-between">
          <div className="space-y-1">
            <p className="text-xs font-semibold uppercase tracking-[0.3em] text-emerald-600">
              Protocol Workbench
            </p>
            <h1 className="text-xl font-semibold tracking-tight text-slate-950">
              HSMS Transport Lab
            </h1>
            <p className="text-sm text-slate-600">
              Explore TCP sessions, HSMS control messages, and SECS-II payload exchange.
            </p>
          </div>
        </header>

        <main className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[280px_minmax(0,1fr)]">
          <aside className="flex min-h-0 flex-col gap-4">
            <WorkspaceNavigationCard
              description="Jump between HSMS transport."
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
