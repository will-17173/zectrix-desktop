import { useEffect, useRef } from "react";
import type { ImageLoopTask } from "@/lib/tauri";

type PushFolderImageFn = (taskId: number) => Promise<ImageLoopTask>;

export function useImageLoopRunner(
  tasks: ImageLoopTask[],
  pushFolderImage: PushFolderImageFn,
  onTaskUpdate: (task: ImageLoopTask) => void,
) {
  const timersRef = useRef<Map<number, NodeJS.Timeout>>(new Map());

  useEffect(() => {
    const runningTasks = tasks.filter((t) => t.status === "running");
    const runningIds = new Set(runningTasks.map((t) => t.id));

    // 清理已停止任务的定时器
    for (const [taskId, timer] of timersRef.current.entries()) {
      if (!runningIds.has(taskId)) {
        clearInterval(timer);
        timersRef.current.delete(taskId);
      }
    }

    // 为运行中的任务创建定时器
    for (const task of runningTasks) {
      if (!timersRef.current.has(task.id)) {
        const timer = setInterval(async () => {
          try {
            const updated = await pushFolderImage(task.id);
            onTaskUpdate(updated);
            if (updated.status !== "running") {
              clearInterval(timer);
              timersRef.current.delete(task.id);
            }
          } catch (e) {
            // 错误已由后端记录，清理定时器
            clearInterval(timer);
            timersRef.current.delete(task.id);
          }
        }, task.intervalSeconds * 1000);

        timersRef.current.set(task.id, timer);
      }
    }

    return () => {
      for (const timer of timersRef.current.values()) {
        clearInterval(timer);
      }
      timersRef.current.clear();
    };
  }, [tasks, pushFolderImage, onTaskUpdate]);
}
