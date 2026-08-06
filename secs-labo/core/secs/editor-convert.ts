import type { Secs2NodeId, Secs2NodeState } from "@/types/editor";
import type { Secs2Item } from "@/types/secs2";

/**
 * node state 정보를 기반으로 secs2item을 생성하는 로직
 * @param state 
 * @param nodeId 
 * @returns 
 */
function buildSecs2Item(state: Secs2NodeState, nodeId: Secs2NodeId): Secs2Item {
  const node = state.nodes[nodeId];

  if (!node) {
    throw new Error(`Editor node not found: ${nodeId}`);
  }

  if (node.value.format === "list") {
    return {
      format: "list",
      value: node.value.children.map((childId) => buildSecs2Item(state, childId)),
    };
  }

  return {
    format: node.value.format,
    value: node.value.value ?? [],
  } as Secs2Item;
}

/**
 * node state을 Secs2Item으로 변환한다.
 * @param state node state
 * @returns secs item 규격
 */
export function toSecs2Item(state: Secs2NodeState): Secs2Item {
  return buildSecs2Item(state, state.rootId);
}
