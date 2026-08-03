"use client";

import init, {
  init_web_logger as _init_wasm_web_logger,
  decode_secs2,
  encode_secs2,
} from "secs-runtime-web";

let wasmLoadPromise: Promise<void> | null = null;


export function initWasm(): Promise<void> {
  if (!wasmLoadPromise) {
    wasmLoadPromise = init()
      .then(() => undefined)
      .catch((error) => {
        wasmLoadPromise = null;
        throw error;
      });
  }

  return wasmLoadPromise;
}

export type WasmLogCallback = (level: string, message: string) => void;

/**
 * rust secs-logger에 대한 web logger 로직을 초기화한다.
 * @param level 허용할 로그 레벨
 * @param callback 메시지 도착 시 호출할 콜백 메서드
 */
export function initWebLogger(
  level: "off" | "error" | "warn" | "info" | "debug" | "trace",
  callback: WasmLogCallback,
) {
  _init_wasm_web_logger(level, callback);
}

export { decode_secs2, encode_secs2 };
