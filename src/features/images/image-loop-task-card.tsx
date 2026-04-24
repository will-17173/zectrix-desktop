import { useState } from "react";
import { confirm } from "@tauri-apps/plugin-dialog";
import type { ImageLoopTask, DeviceRecord } from "../../lib/tauri";

type Props = {
  task: ImageLoopTask;
  devices: DeviceRecord[];
  onStart: (taskId: number) => Promise<void>;
  onStop: (taskId: number) => Promise<void>;
  onEdit: (task: ImageLoopTask) => void;
  onDelete: (taskId: number) => Promise<void>;
};

const STATUS_COLORS: Record<string, string> = {
  idle: "bg-gray-400",
  running: "bg-green-500",
  completed: "bg-yellow-500",
  error: "bg-red-500",
};

const STATUS_TEXT: Record<string, string> = {
  idle: "未启动",
  running: "运行中",
  completed: "已完成",
  error: "错误",
};

export function ImageLoopTaskCard({
  task,
  devices,
  onStart,
  onStop,
  onEdit,
  onDelete,
}: Props) {
  const [expanded, setExpanded] = useState(false);
  const [loading, setLoading] = useState(false);

  const device = devices.find((d) => d.deviceId === task.deviceId);
  const deviceLabel = device ? `${device.alias} (${device.deviceId.slice(0, 8)})` : task.deviceId;

  const handleStart = async () => {
    setLoading(true);
    try {
      await onStart(task.id);
    } finally {
      setLoading(false);
    }
  };

  const handleStop = async () => {
    setLoading(true);
    try {
      await onStop(task.id);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    const confirmed = await confirm(`确定要删除任务 "${task.name}" 吗？`, {
      title: "删除任务",
      kind: "warning",
    });
    if (!confirmed) return;
    await onDelete(task.id);
  };

  const durationLabel = () => {
    if (task.durationType === "none") return "无限制";
    if (task.durationType === "until_time") return `运行至 ${task.endTime}`;
    if (task.durationType === "for_duration") return `运行 ${task.durationMinutes} 分钟`;
    return "";
  };

  const runningInfo = () => {
    if (task.status !== "running" || !task.startedAt) return null;
    const started = new Date(task.startedAt);
    const now = new Date();
    const elapsedMinutes = Math.floor((now.getTime() - started.getTime()) / 60000);
    const rounds = Math.floor(task.currentIndex / task.totalImages);
    return `已运行 ${elapsedMinutes} 分钟，播放 ${rounds} 轮`;
  };

  return (
    <div className="rounded-lg border border-gray-200 bg-white shadow-sm overflow-hidden">
      <div className="p-3 flex items-center gap-3">
        <span className={`w-3 h-3 rounded-full ${STATUS_COLORS[task.status]}`} />
        <span className="font-medium flex-1">任务名称: {task.name}</span>
        <div className="flex gap-2">
          {task.status === "idle" || task.status === "error" ? (
            <button
              type="button"
              onClick={handleStart}
              disabled={loading}
              className="px-2 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
            >
              ▶
            </button>
          ) : task.status === "running" ? (
            <button
              type="button"
              onClick={handleStop}
              disabled={loading}
              className="px-2 py-1 text-sm bg-gray-600 text-white rounded hover:bg-gray-700 disabled:opacity-50"
            >
              ⏹
            </button>
          ) : null}
        </div>
      </div>
      <div className="px-3 pb-2 text-sm text-gray-600">
        {task.status === "running"
          ? `当前: 第 ${task.currentIndex + 1}/${task.totalImages} 张`
          : task.status === "error"
          ? task.errorMessage || "错误"
          : STATUS_TEXT[task.status]}
      </div>
      <button
        type="button"
        onClick={() => setExpanded(!expanded)}
        className="w-full px-3 pb-2 text-sm text-blue-600 hover:text-blue-800"
      >
        {expanded ? "▼ 收起" : "▶ 展开"}
      </button>

      {expanded && (
        <div className="border-t border-gray-200 p-3 text-sm space-y-1">
          <div>文件夹: {task.folderPath}</div>
          <div>目标: {deviceLabel} / 第 {task.pageId} 页</div>
          <div>间隔: {task.intervalSeconds} 秒</div>
          <div>持续: {durationLabel()}</div>
          {runningInfo() && <div>{runningInfo()}</div>}
          {task.lastPushAt && <div>最后推送: {new Date(task.lastPushAt).toLocaleTimeString()}</div>}
          <hr className="my-2 border-gray-200" />
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => onEdit(task)}
              className="px-2 py-1 text-sm border border-gray-300 rounded hover:bg-gray-100"
            >
              编辑
            </button>
            <button
              type="button"
              onClick={handleDelete}
              className="px-2 py-1 text-sm border border-red-300 text-red-600 rounded hover:bg-red-50"
            >
              删除
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
