export type SyncState = "idle" | "syncing" | "success" | "offline" | "error";

export function syncStateLabel(state: SyncState): string {
  const labels: Record<SyncState, string> = {
    idle: "未同步",
    syncing: "同步中",
    success: "已完成",
    offline: "离线",
    error: "失败",
  };
  return labels[state];
}
