"use client";

import { useEffect, useRef, useState } from "react";
import HexEditor from "@/components/editor/hex/HexEditor";
import { useByteEditor } from "@/components/editor/hex/hooks/useEditor";
import { hexRegex } from "@/components/editor/hex/util";
import {
  TransferOutputList,
  type TransferOutputListItem,
} from "@/components/protocols/secs1/TransferOutputList";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { JsSecs1BlockTransfer } from "@/lib/wasm";
import { hexStrToNum, numToHexStr } from "@/lib/convert";
import { useWebLogStore } from "@/stores/weblog/useWebLogStore";
import type { Secs1Block, Secs1BlockHeader, Secs1TransportConfig } from "@/types/secs1";
import { z } from "zod";

const defaultConfig: Secs1TransportConfig = {
  device_id: 1,
  local_role: "Active",
  t1_timeout: "1s",
  t2_timeout: "1s",
  t3_timeout: "45s",
  t4_timeout: "10s",
  t2_rty_limit: 2,
};

const defaultHeader: Secs1BlockHeader = {
  device_id: 1,
  rbit: false,
  wbit: true,
  stream: 1,
  function: 3,
  ebit: true,
  block_no: 1,
  system_byte: 1,
};

const configFormSchema = z
  .object({
    deviceId: z.coerce.number().int(),
    localRole: z.enum(["Active", "Passive"]),
    t1Timeout: z.string().min(1),
    t2Timeout: z.string().min(1),
    t3Timeout: z.string().min(1),
    t4Timeout: z.string().min(1),
    t2RetryLimit: z.coerce.number().int(),
  })
  .transform(
    (form): Secs1TransportConfig => ({
      device_id: form.deviceId,
      local_role: form.localRole,
      t1_timeout: form.t1Timeout,
      t2_timeout: form.t2Timeout,
      t3_timeout: form.t3Timeout,
      t4_timeout: form.t4Timeout,
      t2_rty_limit: form.t2RetryLimit,
    }),
  );

export default function Secs1BlockTransferPage() {
  const transferRef = useRef<JsSecs1BlockTransfer | null>(null);
  const readByteEditor = useByteEditor();
  const writeBlockEditor = useByteEditor();
  const addLogListener = useWebLogStore((store) => store.addListener);
  const logs = useWebLogStore((store) => store.logs);

  const [machineState, setMachineState] = useState("not created");
  const [configOpen, setConfigOpen] = useState(true);
  const [errors, setErrors] = useState<TransferOutputListItem[]>([]);

  const [configForm, setConfigForm] = useState({
    deviceId: String(defaultConfig.device_id),
    localRole: String(defaultConfig.local_role),
    t1Timeout: defaultConfig.t1_timeout,
    t2Timeout: defaultConfig.t2_timeout,
    t3Timeout: defaultConfig.t3_timeout,
    t4Timeout: defaultConfig.t4_timeout,
    t2RetryLimit: String(defaultConfig.t2_rty_limit),
  });

  const [blockHeaderForm, setBlockHeaderForm] = useState({
    deviceId: String(defaultHeader.device_id),
    rbit: String(defaultHeader.rbit),
    wbit: String(defaultHeader.wbit),
    stream: String(defaultHeader.stream),
    function: String(defaultHeader.function),
    ebit: String(defaultHeader.ebit),
    blockNo: String(defaultHeader.block_no),
    systemByte: String(defaultHeader.system_byte),
  });
  const [timeoutForm, setTimeoutForm] = useState({
    id: "",
    unit: "t2",
  });

  const [writeOutputs, setWriteOutputs] = useState<TransferOutputListItem[]>([]);
  const [readOutputs, setReadOutputs] = useState<TransferOutputListItem[]>([]);
  const [timeoutOutputs, setTimeoutOutputs] = useState<TransferOutputListItem[]>([]);
  const [eventOutputs, setEventOutputs] = useState<TransferOutputListItem[]>([]);

  useEffect(() => {
    writeBlockEditor.setBytes([0x01, 0x01, 0xb1, 0x04, 0x00, 0x00, 0x0b, 0xb8]);
  }, []);

  useEffect(() => {
    return () => {
      transferRef.current?.free();
      transferRef.current = null;
    };
  }, []);

  useEffect(() => {
    const applyStateChangeLog = (log: { level: string; msg: string }) => {
      if (log.level.toUpperCase() !== "DEBUG") return;

      const matched = log.msg.match(/state change:\s+(.+?)\s+->\s+(.+)$/);
      if (!matched) return;

      setMachineState(matched[2]);
    };

    logs.forEach(applyStateChangeLog);
    return addLogListener(applyStateChangeLog);
  }, [addLogListener, logs]);

  const addError = (message: string) => {
    setErrors((current) => [...current, { id: current.length + 1, value: message }]);
  };

  const pollAll = (transfer: JsSecs1BlockTransfer) => {
    const nextWrites: string[] = [];
    const nextReads: string[] = [];
    const nextTimeouts: string[] = [];
    const nextEvents: string[] = [];

    for (; ;) {
      const bytes = transfer.poll_write();
      if (!bytes) break;
      nextWrites.push(
        Array.from(bytes)
          .map((byte) => byte.toString(16).padStart(2, "0").toUpperCase())
          .join(" "),
      );
    }

    for (; ;) {
      const block = transfer.poll_read();
      if (!block) break;
      try {
        nextReads.push(JSON.stringify(JSON.parse(block), null, 2));
      } catch {
        nextReads.push(block);
      }
    }

    for (; ;) {
      const timeout = transfer.poll_timeout();
      if (!timeout) break;
      nextTimeouts.push(timeout);
    }

    for (; ;) {
      const event = transfer.poll_event();
      if (!event) break;
      nextEvents.push(event);
    }

    if (nextWrites.length > 0) {
      setWriteOutputs((current) => [
        ...current,
        ...nextWrites.map((value, index) => ({ id: current.length + index + 1, value })),
      ]);
    }
    if (nextReads.length > 0) {
      setReadOutputs((current) => [
        ...current,
        ...nextReads.map((value, index) => ({ id: current.length + index + 1, value })),
      ]);
    }
    if (nextTimeouts.length > 0) {
      setTimeoutOutputs((current) => [
        ...current,
        ...nextTimeouts.map((value, index) => ({ id: current.length + index + 1, value })),
      ]);

      const latest = nextTimeouts[nextTimeouts.length - 1];
      try {
        const key = JSON.parse(latest) as { id: number; unit: string };
        setTimeoutForm({
          id: String(key.id),
          unit: key.unit,
        });
      } catch {
        setTimeoutForm((current) => ({ ...current, id: "" }));
      }
    }
    if (nextEvents.length > 0) {
      setEventOutputs((current) => [
        ...current,
        ...nextEvents.map((value, index) => ({ id: current.length + index + 1, value })),
      ]);
    }
  };

  const withTransfer = (task: (transfer: JsSecs1BlockTransfer) => void) => {
    const transfer = transferRef.current;
    if (!transfer) {
      addError("transfer is not created");
      return;
    }

    try {
      task(transfer);
      pollAll(transfer);
    } catch (error) {
      addError(String(error));
    }
  };

  const createTransfer = () => {
    try {
      const config = configFormSchema.parse(configForm);

      transferRef.current?.free();
      transferRef.current = new JsSecs1BlockTransfer(JSON.stringify(config));
      setMachineState("created");
      setErrors([]);
      setWriteOutputs([]);
      setReadOutputs([]);
      setTimeoutOutputs([]);
      setEventOutputs([]);
    } catch (error) {
      addError(`create failed: ${String(error)}`);
    }
  };

  const clearTransfer = () => {
    transferRef.current?.free();
    transferRef.current = null;
    setMachineState("not created");
    setErrors([]);
    setWriteOutputs([]);
    setReadOutputs([]);
    setTimeoutOutputs([]);
    setEventOutputs([]);
  };

  const handleRead = () => {
    withTransfer((transfer) => {
      transfer.read(Uint8Array.from(readByteEditor.bytes));
    });
  };

  const handleWrite = () => {
    withTransfer((transfer) => {
      const block: Secs1Block = {
        header: {
          device_id: Number(blockHeaderForm.deviceId),
          rbit: blockHeaderForm.rbit === "true",
          wbit: blockHeaderForm.wbit === "true",
          stream: Number(blockHeaderForm.stream),
          function: Number(blockHeaderForm.function),
          ebit: blockHeaderForm.ebit === "true",
          block_no: Number(blockHeaderForm.blockNo),
          system_byte: Number(blockHeaderForm.systemByte),
        },
        data: writeBlockEditor.bytes,
      };
      transfer.write(JSON.stringify(block));
    });
  };

  const handleTimeout = () => {
    withTransfer((transfer) => {
      const id = Number(timeoutForm.id);
      if (!Number.isInteger(id)) throw new Error("timeout id must be an integer");
      transfer.timeout(JSON.stringify({ id, unit: timeoutForm.unit }));
    });
  };

  const machineCreated = machineState !== "not created";
  const machineTone = !machineCreated
    ? "border-slate-700 bg-slate-800 text-slate-100"
    : machineState.includes("Invalid")
      ? "border-red-300 bg-red-50 text-red-700"
      : "border-emerald-200 bg-white text-slate-950";

  return (
    <div className="flex min-h-0 flex-col gap-4">
      <Card className="bg-white">
        <CardHeader>
          <div>
            <CardTitle>SECS-I Block Transfer</CardTitle>
            <CardDescription>
              Configure a WASM-backed transfer machine, push inputs from both layers, and inspect
              every poll output without overwriting previous results.
            </CardDescription>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid gap-3 xl:grid-cols-[minmax(300px,0.9fr)_minmax(280px,0.75fr)_minmax(320px,0.95fr)]">
            <section className="flex min-h-0 flex-col gap-4">
              <Card className="bg-white">
                <CardHeader>
                  <CardTitle>handle_read</CardTitle>
                  <CardDescription>Lower layer bytes.</CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  <HexEditor
                    name="READ HEX"
                    validator={hexRegex}
                    slotPerLine={8}
                    charPerSlot={2}
                    parseFunc={hexStrToNum}
                    displayFunc={numToHexStr}
                    aria-label="SECS1_READ_HEX"
                    {...readByteEditor}
                  />
                  <Button size="sm" onClick={handleRead}>
                    OK
                  </Button>
                </CardContent>
              </Card>

              <Card className="bg-white">
                <CardHeader>
                  <CardTitle>handle_write</CardTitle>
                  <CardDescription>Upper layer block form.</CardDescription>
                </CardHeader>
                <CardContent className="grid gap-3">
                  <div className="grid grid-cols-2 gap-3">
                    <Field label="device_id">
                      <Input
                        value={blockHeaderForm.deviceId}
                        onChange={(event) =>
                          setBlockHeaderForm((current) => ({
                            ...current,
                            deviceId: event.target.value,
                          }))
                        }
                      />
                    </Field>
                    <Field label="rbit">
                      <Input
                        value={blockHeaderForm.rbit}
                        onChange={(event) =>
                          setBlockHeaderForm((current) => ({ ...current, rbit: event.target.value }))
                        }
                      />
                    </Field>
                    <Field label="wbit">
                      <Input
                        value={blockHeaderForm.wbit}
                        onChange={(event) =>
                          setBlockHeaderForm((current) => ({ ...current, wbit: event.target.value }))
                        }
                      />
                    </Field>
                    <Field label="stream">
                      <Input
                        value={blockHeaderForm.stream}
                        onChange={(event) =>
                          setBlockHeaderForm((current) => ({ ...current, stream: event.target.value }))
                        }
                      />
                    </Field>
                    <Field label="function">
                      <Input
                        value={blockHeaderForm.function}
                        onChange={(event) =>
                          setBlockHeaderForm((current) => ({
                            ...current,
                            function: event.target.value,
                          }))
                        }
                      />
                    </Field>
                    <Field label="ebit">
                      <Input
                        value={blockHeaderForm.ebit}
                        onChange={(event) =>
                          setBlockHeaderForm((current) => ({ ...current, ebit: event.target.value }))
                        }
                      />
                    </Field>
                    <Field label="block_no">
                      <Input
                        value={blockHeaderForm.blockNo}
                        onChange={(event) =>
                          setBlockHeaderForm((current) => ({
                            ...current,
                            blockNo: event.target.value,
                          }))
                        }
                      />
                    </Field>
                    <Field label="system_byte">
                      <Input
                        value={blockHeaderForm.systemByte}
                        onChange={(event) =>
                          setBlockHeaderForm((current) => ({
                            ...current,
                            systemByte: event.target.value,
                          }))
                        }
                      />
                    </Field>
                  </div>
                  <HexEditor
                    name="DATA HEX"
                    validator={hexRegex}
                    slotPerLine={8}
                    charPerSlot={2}
                    parseFunc={hexStrToNum}
                    displayFunc={numToHexStr}
                    aria-label="SECS1_WRITE_DATA_HEX"
                    {...writeBlockEditor}
                  />
                  <Button size="sm" onClick={handleWrite}>
                    OK
                  </Button>
                </CardContent>
              </Card>

              <Card className="bg-white">
                <CardHeader>
                  <CardTitle>handle_timeout</CardTitle>
                  <CardDescription>Use a timeout key returned by poll_timeout.</CardDescription>
                </CardHeader>
                <CardContent className="grid gap-3">
                  <div className="grid grid-cols-2 gap-3">
                    <Field label="id">
                      <Input
                        value={timeoutForm.id}
                        onChange={(event) =>
                          setTimeoutForm((current) => ({ ...current, id: event.target.value }))
                        }
                      />
                    </Field>
                    <Field label="unit">
                      <Input
                        value={timeoutForm.unit}
                        onChange={(event) =>
                          setTimeoutForm((current) => ({ ...current, unit: event.target.value }))
                        }
                      />
                    </Field>
                  </div>
                  <Button size="sm" onClick={handleTimeout}>
                    OK
                  </Button>
                </CardContent>
              </Card>
            </section>

            <section className="relative self-start">
              <Card className="sticky top-2 max-h-[calc(100vh-1rem)] overflow-auto border-emerald-200 bg-[radial-gradient(circle_at_top,_rgba(16,185,129,0.16),_transparent_52%),linear-gradient(180deg,_#ffffff_0%,_#ecfdf5_100%)] shadow-sm">
                <CardHeader className="text-center">
                  <CardTitle>Secs1Machine</CardTitle>
                  <CardDescription>WASM transfer object boundary</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4 text-center">
                  <div className={`mx-auto flex size-44 flex-col items-center justify-center rounded-full border px-4 shadow-inner ${machineTone}`}>
                    <span className="text-[0.65rem] font-semibold uppercase tracking-[0.22em] opacity-70">
                      {machineCreated ? "created" : "empty"}
                    </span>
                    <span className="mt-2 max-w-36 break-words text-center font-mono text-xs font-semibold leading-4">
                      {machineState}
                    </span>
                  </div>
                  <div className="rounded-xl border border-emerald-200 bg-white/85 p-3 text-left">
                    <div className="mb-3 flex items-start justify-between gap-2">
                      <div>
                        <p className="font-semibold text-slate-950">Config</p>
                        <p className="text-xs text-slate-500">
                          적용 시 transfer object를 새로 생성합니다.
                        </p>
                      </div>
                      <Button
                        type="button"
                        size="xs"
                        variant="outline"
                        onClick={() => setConfigOpen((current) => !current)}
                      >
                        {configOpen ? "닫기" : "열기"}
                      </Button>
                    </div>
                    {configOpen ? (
                    <div className="grid gap-2">
                      <Field label="device_id">
                        <Input
                          value={configForm.deviceId}
                          onChange={(event) =>
                            setConfigForm((current) => ({
                              ...current,
                              deviceId: event.target.value,
                            }))
                          }
                        />
                      </Field>
                      <Field label="local_role">
                        <Input
                          value={configForm.localRole}
                          onChange={(event) =>
                            setConfigForm((current) => ({
                              ...current,
                              localRole: event.target.value,
                            }))
                          }
                        />
                      </Field>
                      <div className="grid grid-cols-2 gap-2">
                        <Field label="t1">
                          <Input
                            value={configForm.t1Timeout}
                            onChange={(event) =>
                              setConfigForm((current) => ({
                                ...current,
                                t1Timeout: event.target.value,
                              }))
                            }
                          />
                        </Field>
                        <Field label="t2">
                          <Input
                            value={configForm.t2Timeout}
                            onChange={(event) =>
                              setConfigForm((current) => ({
                                ...current,
                                t2Timeout: event.target.value,
                              }))
                            }
                          />
                        </Field>
                        <Field label="t3">
                          <Input
                            value={configForm.t3Timeout}
                            onChange={(event) =>
                              setConfigForm((current) => ({
                                ...current,
                                t3Timeout: event.target.value,
                              }))
                            }
                          />
                        </Field>
                        <Field label="t4">
                          <Input
                            value={configForm.t4Timeout}
                            onChange={(event) =>
                              setConfigForm((current) => ({
                                ...current,
                                t4Timeout: event.target.value,
                              }))
                            }
                          />
                        </Field>
                      </div>
                      <Field label="t2_rty_limit">
                        <Input
                          value={configForm.t2RetryLimit}
                          onChange={(event) =>
                            setConfigForm((current) => ({
                              ...current,
                              t2RetryLimit: event.target.value,
                            }))
                          }
                        />
                      </Field>
                      <div className="grid grid-cols-2 gap-2">
                        <Button size="sm" onClick={createTransfer}>
                          적용
                        </Button>
                        <Button size="sm" variant="outline" onClick={clearTransfer}>
                          clear
                        </Button>
                      </div>
                    </div>
                    ) : null}
                  </div>
                  {errors.length > 0 ? (
                    <TransferOutputList
                      title="Errors"
                      description="Operation failures."
                      items={errors}
                    />
                  ) : null}
                </CardContent>
              </Card>
            </section>

            <section className="grid min-h-0 gap-4">
              <TransferOutputList
                title="poll_write"
                description="Lower layer byte output."
                items={writeOutputs}
              />
              <TransferOutputList
                title="poll_read"
                description="Upper layer block JSON output."
                items={readOutputs}
              />
              <TransferOutputList
                title="poll_timeout"
                description="Timeout key JSON output."
                items={timeoutOutputs}
              />
              <TransferOutputList
                title="poll_event"
                description="Debug event string output."
                items={eventOutputs}
              />
            </section>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="grid gap-1.5">
      <Label className="font-mono text-xs text-slate-600">{label}</Label>
      {children}
    </div>
  );
}
