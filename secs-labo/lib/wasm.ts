"use client";

import { LogLevel } from "@/types/log";
import init, {
  JsSecs1BlockTransfer,
  init_web_logger as _init_wasm_web_logger,
  decode_secs2,
  encode_secs2,
} from "@/wasm/secs-runtime-web/secs_runtime_web";

let wasmLoadPromise: Promise<void> | null = null;

/**
 * secs-web 프로젝트 wasm 초기화
 * @returns 
 */
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
  level: LogLevel,
  callback: WasmLogCallback,
) {
  _init_wasm_web_logger(level, callback);
  console.info("wasm web logger initialized");
}

export { JsSecs1BlockTransfer, decode_secs2, encode_secs2 };
