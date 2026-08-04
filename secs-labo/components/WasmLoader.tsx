"use client";

import { useEffect } from "react";
import { initWasm, initWebLogger } from "@/lib/wasm";
import { useWebLogStore } from "@/stores/weblog/useWebLogStore";

export function WasmLoader() {
  const addLog = useWebLogStore(it => it.add);

  useEffect(() => {
    initWasm()
      .then((_) => {
        console.info("success to load wasm");
        // logger 초기화
        initWebLogger("debug", (level, msg) => {
          addLog({ level, msg });
        });
      })
      .catch((error) => {
        console.error("failed to load wasm", error);
      });

  }, []);

  return null;
}
