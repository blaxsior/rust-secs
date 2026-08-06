import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Cable, Clock, Workflow } from "lucide-react";

const sections = [
  {
    title: "Serial Transport",
    description: "Review SECS-I connection settings such as serial port, baud rate, retry behavior, and block handling.",
    icon: Cable,
  },
  {
    title: "Timeout Flow",
    description: "Track T1/T2/T4 style timing concerns for serial message exchange and retry decisions.",
    icon: Clock,
  },
  {
    title: "Message Bridge",
    description: "Prepare SECS-I transport flows that can share SECS-II message encoding and decoding tools.",
    icon: Workflow,
  },
] as const;

export default function Secs1Page() {
  return (
    <div className="flex min-h-0 flex-col gap-4">
      <Card className="bg-white">
        <CardHeader>
          <CardTitle>SECS-I Overview</CardTitle>
          <CardDescription>
            Draft workspace for serial SECS-I transport inspection and message flow planning.
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
