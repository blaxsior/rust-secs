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
    description: "Raw bytes(bin/hex) into SECS-II Message",
    icon: Binary,
  },
  {
    title: "Message to Byte",
    description: "SECS-II Message to Raw bytes(bin/hex)",
    icon: FileCode2,
  }
];

export default function Secs2Page() {
  return (
    <div className="flex min-h-0 flex-col gap-4">
      <div className="flex flex-col xl:flex-row gap-4">
        {workflowCards.map((card) => {
          const Icon = card.icon;
          return (
            <Card key={card.title} className="bg-white flex-1">
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
            Explore SECS-II messages from both byte-level and structured message-level workflows.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-3 md:grid-cols-3">
          <Card>
            <CardHeader>
              <CardTitle>Byte to Message</CardTitle>
              <CardDescription>
                Write raw binary data in either binary or hexadecimal form, then decode it into a structured SECS-II message.
              </CardDescription>
            </CardHeader>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Message to Byte</CardTitle>
              <CardDescription>
                Build a SECS-II message with an editor-driven workflow, then inspect the encoded byte output in binary and hexadecimal views.
              </CardDescription>
            </CardHeader>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Web Log</CardTitle>
              <CardDescription>
                Review logs emitted by the Rust library during SECS-II parse and serialize operations directly in the web workspace.
              </CardDescription>
            </CardHeader>
          </Card>
        </CardContent>
      </Card>
    </div>
  );
}
