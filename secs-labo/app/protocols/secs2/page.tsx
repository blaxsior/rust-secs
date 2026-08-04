import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Binary, FileCode2, Workflow } from "lucide-react";

const workflowCards = [
  {
    title: "Byte to Message",
    description: "Raw bytes into SECS-II structure.",
    icon: Binary,
  },
  {
    title: "Message to Byte",
    description: "Structured fields back to payload bytes.",
    icon: FileCode2,
  },
  {
    title: "Runtime Preview",
    description: "Bridge point for the Rust-based simulator.",
    icon: Workflow,
  },
];

export default function Secs2Page() {
  return (
    <div className="flex min-h-0 flex-col gap-4">
      <div className="grid gap-4 md:grid-cols-3">
        {workflowCards.map((card) => {
          const Icon = card.icon;
          return (
            <Card key={card.title} className="bg-white">
              <CardContent className="flex items-start justify-between gap-3 p-4">
                <div>
                  <p className="text-sm font-semibold text-slate-950">{card.title}</p>
                  <p className="mt-2 text-sm leading-6 text-slate-600">{card.description}</p>
                </div>
                <div className="rounded-xl bg-slate-100 p-2 text-slate-700">
                  <Icon className="size-4" />
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>

      <Card className="bg-white">
        <CardHeader>
          <CardTitle>SECS-II Overview</CardTitle>
          <CardDescription>
            Choose encode or decode from the left navigation to enter a dedicated workspace.
          </CardDescription>
        </CardHeader>
        <CardContent className="text-sm leading-6 text-slate-600">
          The overview page is intentionally light. Detailed editing now lives in the
          dedicated subpages so the main workspace can stay focused on a single job.
        </CardContent>
      </Card>
    </div>
  );
}
