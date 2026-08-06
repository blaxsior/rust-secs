import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Network, RadioTower, Workflow } from "lucide-react";

const sections = [
  {
    title: "TCP Session",
    description: "Inspect active/passive HSMS endpoint settings and one-connection-per-session assumptions.",
    icon: Network,
  },
  {
    title: "Control Messages",
    description: "Prepare Select, Linktest, Separate, and timeout handling views for HSMS control flow.",
    icon: RadioTower,
  },
  {
    title: "SECS-II Payload",
    description: "Reuse SECS-II encode/decode workflows while keeping HSMS transport state separate.",
    icon: Workflow,
  },
] as const;

export default function HsmsPage() {
  return (
    <div className="flex min-h-0 flex-col gap-4">
      <Card className="bg-white">
        <CardHeader>
          <CardTitle>HSMS Overview</CardTitle>
          <CardDescription>
            Draft workspace for HSMS TCP connection state, control messages, and SECS-II payload exchange.
          </CardDescription>
        </CardHeader>
      </Card>

      <section className="grid gap-4 md:grid-cols-3">
        {sections.map((section) => {
          const Icon = section.icon;

          return (
            <Card key={section.title} className="bg-white">
              <CardContent className="flex items-start justify-between gap-3 p-4">
                <div>
                  <p className="text-sm font-semibold text-slate-950">{section.title}</p>
                  <p className="mt-2 text-sm leading-6 text-slate-600">{section.description}</p>
                </div>
                <div className="rounded-xl bg-slate-100 p-2 text-slate-700">
                  <Icon className="size-4" />
                </div>
              </CardContent>
            </Card>
          );
        })}
      </section>
    </div>
  );
}
