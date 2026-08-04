import { LogLevel } from "@/types/log";
import { create } from "zustand";

export type WebLogLine = {
  level: string;
  msg: string;
};

export type WebLogState = {
  /**
   * log 최대 라인 수
   */
  limit: number;
  /**
   * 저장된 로그 정보
   */
  logs: WebLogLine[];
};

type WebLogAction = {
  /**
   * log 메시지를 추가한다.
   * @param log 정보
   */
  add: (log: WebLogLine) => void;
  /**
   * log 내용을 초기화한다.
   */
  clear: () => void;

  /**
   * log line limit를 설정
   * @param limit 로그 최대 라인 수
   * @returns
   */
  setLineLimit: (limit: number) => void;
};

type WebLogStore = WebLogState & WebLogAction;

export const useWebLogStore = create<WebLogStore>((set, get) => ({
  level: "debug",
  limit: 100,
  logs: [],
  add: (log: WebLogLine) => set((state) => {
      const newlogs = [...state.logs, log];
      return { logs: newlogs.slice(-state.limit) };
  }),
  clear: () => set((_) => ({ logs: [] })),
  setLineLimit: (limit: number) => set(() => ({ limit: limit })),
}));
