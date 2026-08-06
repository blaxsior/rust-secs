/**
 * context menu close보다 늦게 action을 실행하기 위한 메서드
 * @param action 
 */
export function runAfterContextMenuClose(action: () => void) {
  window.setTimeout(action, 10);
}
