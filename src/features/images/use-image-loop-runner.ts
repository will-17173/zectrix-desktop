import { useEffect, useRef } from "react";
import type { ImageLoopTask } from "../../lib/tauri";

/**
 * Hook to poll for running task status updates.
 * The actual image pushing is now handled by Rust backend background tasks.
 * This hook only triggers periodic refresh of the task list to show status updates.
 */
export function useImageLoopRunner(
  tasks: ImageLoopTask[],
  refreshTasks: () => Promise<void>,
) {
  const pollIntervalRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    const runningTasks = tasks.filter((t) => t.status === "running");

    // Clear poll interval if no running tasks
    if (runningTasks.length === 0) {
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current);
        pollIntervalRef.current = null;
      }
      return;
    }

    // Poll every 5 seconds to refresh task status from backend
    if (!pollIntervalRef.current) {
      pollIntervalRef.current = setInterval(() => {
        refreshTasks().catch((e) => {
          console.error("[image-loop] 刷新任务状态失败:", e);
        });
      }, 5000);
    }

    return () => {
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current);
        pollIntervalRef.current = null;
      }
    };
  }, [tasks, refreshTasks]);
}