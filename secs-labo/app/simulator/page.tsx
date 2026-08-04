import Link from "next/link";
import { Cpu, ArrowRight } from "lucide-react";

export default function SimulatorPage() {
  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,_rgba(249,115,22,0.12),_transparent_28%),linear-gradient(180deg,_#fafafa_0%,_#f3f4f6_100%)] px-4 py-8 text-slate-900 sm:px-6 lg:px-8">
      <div className="mx-auto flex w-full max-w-5xl flex-col gap-6">
        <section className="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm">
          <div className="flex items-start justify-between gap-4">
            <div>
              <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
                Simulator
              </p>
              <h1 className="mt-2 text-3xl font-semibold tracking-tight text-slate-950">
                Runtime entry point
              </h1>
              <p className="mt-3 max-w-2xl text-sm leading-7 text-slate-600 sm:text-base">
                Rust 기반 통신 라이브러리를 연결해서 실제 메시지 흐름을 재현할 수 있는
                공간입니다. 지금은 시작점만 잡아둔 상태입니다.
              </p>
            </div>

            <div className="rounded-2xl border border-slate-200 bg-slate-950 p-3 text-white">
              <Cpu className="size-6" />
            </div>
          </div>

          <div className="mt-6 flex flex-wrap gap-3">
            <Link
              href="/"
              className="inline-flex items-center gap-2 rounded-full bg-slate-950 px-4 py-2.5 text-sm font-medium text-white transition-colors hover:bg-slate-800"
            >
              Back to home
              <ArrowRight className="size-4" />
            </Link>
          </div>
        </section>
      </div>
    </main>
  );
}
