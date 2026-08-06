import Link from "next/link";
import { ArrowRight, Binary, Cable, Cpu, Network, Workflow } from "lucide-react";

const sections = [
  {
    title: "Protocol",
    description: "SECS-I, SECS-II, HSMS 메시지를 byte와 구조화 메시지로 변환하고 검증하는 작업 공간입니다.",
    accent: "from-emerald-500/15 to-cyan-500/10",
    items: [
      { label: "SECS-I lab", href: "/protocols/secs1", icon: Cable },
      { label: "SECS-II lab", href: "/protocols/secs2", icon: Binary },
      { label: "HSMS lab", href: "/protocols/hsms", icon: Network },
    ],
  },
  {
    title: "Simulator",
    description: "Rust로 빌드한 통신 라이브러리를 붙여 실제 흐름을 재현하는 실행 공간입니다.",
    accent: "from-amber-500/15 to-orange-500/10",
    items: [
      { label: "Simulator home", href: "/simulator", icon: Cpu },
    ],
  },
];

export default function Home() {
  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,_rgba(14,165,233,0.10),_transparent_30%),linear-gradient(180deg,_#f8fafc_0%,_#eef2f7_100%)] px-4 py-8 text-slate-900 sm:px-6 lg:px-8">
      <div className="mx-auto flex w-full max-w-6xl flex-col gap-6">
        <header className="rounded-3xl border border-slate-200/80 bg-white/80 px-6 py-6 shadow-sm backdrop-blur">
          <p className="text-xs font-semibold uppercase tracking-[0.35em] text-slate-500">
            secs-labo
          </p>
          <h1 className="mt-2 text-3xl font-semibold tracking-tight text-slate-950 sm:text-4xl">
            Message lab and simulator workspace
          </h1>
          <p className="mt-3 max-w-2xl text-sm leading-7 text-slate-600 sm:text-base">
            프로토콜 편집과 시뮬레이션 실행을 분리해서 시작하는 진입 화면입니다.
            아래 버튼으로 바로 각 영역으로 이동할 수 있습니다.
          </p>
        </header>

        <section className="grid gap-4 lg:grid-cols-2">
          {sections.map((section) => (
            <div
              key={section.title}
              className={`rounded-3xl border border-slate-200 bg-gradient-to-br ${section.accent} bg-white p-5 shadow-sm`}
            >
              <div className="flex items-center justify-between gap-3">
                <div>
                  <p className="text-xs font-semibold uppercase tracking-[0.24em] text-slate-500">
                    {section.title}
                  </p>
                  <p className="mt-2 max-w-xl text-sm leading-6 text-slate-600">
                    {section.description}
                  </p>
                </div>
                <div className="rounded-2xl border border-slate-200 bg-white p-3 text-slate-700 shadow-sm">
                  {section.title === "Protocol" ? (
                    <Workflow className="size-5" />
                  ) : (
                    <Cpu className="size-5" />
                  )}
                </div>
              </div>

              <div className="mt-5 flex flex-wrap gap-3">
                {section.items.map((item) => {
                  const Icon = item.icon;
                  return (
                    <Link
                      key={item.href}
                      href={item.href}
                      className="inline-flex items-center gap-2 rounded-full bg-slate-950 px-4 py-2.5 text-sm font-medium text-white transition-colors hover:bg-slate-800"
                    >
                      <Icon className="size-4" />
                      {item.label}
                      <ArrowRight className="size-4" />
                    </Link>
                  );
                })}
              </div>
            </div>
          ))}
        </section>
      </div>
    </main>
  );
}
