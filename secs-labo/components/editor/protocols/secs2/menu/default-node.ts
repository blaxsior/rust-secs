import type { Secs2NodeInput } from "@/types/editor"

export function createDefaultChildNode(): Secs2NodeInput {
  return {
    format: "ascii",
    name: undefined,
    description: undefined,
    value: {
      format: "ascii",
      value: "",
    },
  }
}
