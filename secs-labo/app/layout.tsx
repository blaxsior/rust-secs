import type { Metadata } from "next";
import Link from "next/link";
import { Geist, Geist_Mono } from "next/font/google";
import { Menu, Cpu, Binary, Workflow } from "lucide-react";
import { WasmLoader } from "@/components/WasmLoader";
import "./globals.css";
import { Toaster } from "@/components/ui/toast";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "secs-labo",
  description: "SECS protocol workbench and simulator",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col">
        <WasmLoader />
        <div className="relative min-h-screen">
          <details className="group absolute right-4 top-4 z-50 sm:right-6 sm:top-6">
            <summary className="flex cursor-pointer list-none items-center justify-center rounded-full border border-slate-200 bg-white/90 p-3 text-slate-700 shadow-sm backdrop-blur transition-colors hover:bg-slate-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-slate-400 [&::-webkit-details-marker]:hidden">
              <Menu className="size-5" />
            </summary>

            <div className="absolute right-0 mt-3 w-64 overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-xl">
              <div className="border-b border-slate-100 px-4 py-3">
                <p className="text-xs font-semibold uppercase tracking-[0.24em] text-slate-500">
                  Workspace
                </p>
                <p className="mt-1 text-sm font-medium text-slate-950">
                  Navigate protocol and simulator
                </p>
              </div>

              <div className="p-2">
                <Link
                  href="/"
                  className="flex items-center gap-3 rounded-xl px-3 py-2 text-sm text-slate-700 transition-colors hover:bg-slate-100"
                >
                  <Workflow className="size-4 text-slate-500" />
                  Home
                </Link>
                <Link
                  href="/protocols/secs2"
                  className="flex items-center gap-3 rounded-xl px-3 py-2 text-sm text-slate-700 transition-colors hover:bg-slate-100"
                >
                  <Binary className="size-4 text-slate-500" />
                  SECS-II lab
                </Link>
                <Link
                  href="/simulator"
                  className="flex items-center gap-3 rounded-xl px-3 py-2 text-sm text-slate-700 transition-colors hover:bg-slate-100"
                >
                  <Cpu className="size-4 text-slate-500" />
                  Simulator
                </Link>
              </div>
            </div>
          </details>

          {children}
          <Toaster />
        </div>
      </body>
    </html>
  );
}
