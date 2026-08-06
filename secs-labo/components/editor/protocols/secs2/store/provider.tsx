import { createContext, useContext, useState } from "react";
import { createSecs2EditorStore, Secs2EditorState, Secs2EditorStore } from "./store";
import { StoreApi, UseBoundStore } from "zustand";



// secs2 editor 단위로 상태를 관리하기 위해 context 정의
// editor 자체는 추후 modeling 기능 도입 시 재사용되어야 할 수 있음
type Secs2EditorStoreType = UseBoundStore<StoreApi<Secs2EditorStore>>;
type Secs2EditorSelector<T> = (state: Secs2EditorStore) => T;

const Secs2EditorContext = createContext<Secs2EditorStoreType | null>(null);

export function Secs2EditorProvider({ initState, children }: { initState?: Secs2EditorState, children: React.ReactNode }) {
    const [store] = useState(() => createSecs2EditorStore(initState));

    return (
        <Secs2EditorContext.Provider value={store}>
            {children}
        </Secs2EditorContext.Provider>
    );
}

// secs2 editor store를 사용하기 위한 hook
export function useSecs2EditorStore(): Secs2EditorStore;
export function useSecs2EditorStore<T>(selector: Secs2EditorSelector<T>): T;
export function useSecs2EditorStore<T>(selector?: Secs2EditorSelector<T>) {
  const store = useContext(Secs2EditorContext);

  if (!store) {
    throw new Error("Secs2EditorProvider is not found");
  }

  if (!selector) {
    return store();
  }

  return store(selector);
}
