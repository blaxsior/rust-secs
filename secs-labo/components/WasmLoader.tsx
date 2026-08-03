"use client";

import { useEffect } from "react";
import { initWasm } from "@/lib/wasm";
import { init_web_logger } from "@/wasm/secs-runtime-web";

export function WasmLoader() {
  useEffect(() => {
    initWasm().catch((error) => {
      console.error("failed to load wasm", error);
    });
    init_web_logger("debug", () => {

    });
  }, []);

  return null;
}
