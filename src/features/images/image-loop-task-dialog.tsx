import { useState, useEffect } from "react";
import type { ImageLoopTask, ImageLoopTaskInput, DeviceRecord, ImageFolderScanResult } from "../../lib/tauri";
import { scanImageFolder, selectFolderDialog } from "../../lib/tauri";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";

type Props = {
  open: boolean;
  devices: DeviceRecord[];
  editingTask?: ImageLoopTask;
  onSave: (input: ImageLoopTaskInput) => Promise<void>;
  onClose: () => void;
};

const INTERVAL_OPTIONS = [
  { value: 10, label: "10 秒" },
  { value: 30, label: "30 秒" },
  { value: 60, label: "1 分钟" },
  { value: 300, label: "5 分钟" },
  { value: 600, label: "10 分钟" },
];

const PAGE_OPTIONS = [
  { value: 1, label: "第 1 页" },
  { value: 2, label: "第 2 页" },
  { value: 3, label: "第 3 页" },
  { value: 4, label: "第 4 页" },
  { value: 5, label: "第 5 页" },
];

export function ImageLoopTaskDialog({
  open,
  devices,
  editingTask,
  onSave,
  onClose,
}: Props) {
  const [name, setName] = useState("");
  const [folderPath, setFolderPath] = useState("");
  const [deviceId, setDeviceId] = useState("");
  const [pageId, setPageId] = useState(1);
  const [intervalSeconds, setIntervalSeconds] = useState(30);
  const [durationType, setDurationType] = useState<"none" | "until_time" | "for_duration">("none");
  const [endTime, setEndTime] = useState("");
  const [durationMinutes, setDurationMinutes] = useState(60);
  const [scanResult, setScanResult] = useState<ImageFolderScanResult | null>(null);
  const [saving, setSaving] = useState(false);

  // 生成 datetime-local input 的最小值（当前时间，格式：YYYY-MM-DDTHH:mm）
  const getMinDateTime = () => {
    const now = new Date();
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, "0");
    const day = String(now.getDate()).padStart(2, "0");
    const hours = String(now.getHours()).padStart(2, "0");
    const minutes = String(now.getMinutes()).padStart(2, "0");
    return `${year}-${month}-${day}T${hours}:${minutes}`;
  };

  useEffect(() => {
    if (editingTask) {
      setName(editingTask.name);
      setFolderPath(editingTask.folderPath);
      setDeviceId(editingTask.deviceId);
      setPageId(editingTask.pageId);
      setIntervalSeconds(editingTask.intervalSeconds);
      setDurationType(editingTask.durationType);
      setEndTime(editingTask.endTime || "");
      setDurationMinutes(editingTask.durationMinutes || 60);
      scanFolder(editingTask.folderPath);
    } else {
      resetForm();
    }
  }, [editingTask, open]);

  const resetForm = () => {
    setName("");
    setFolderPath("");
    setDeviceId(devices[0]?.deviceId || "");
    setPageId(1);
    setIntervalSeconds(30);
    setDurationType("none");
    setEndTime("");
    setDurationMinutes(60);
    setScanResult(null);
  };

  const handleSelectFolder = async () => {
    const path = await selectFolderDialog();
    if (path) {
      setFolderPath(path);
      await scanFolder(path);
    }
  };

  const scanFolder = async (path: string) => {
    try {
      const result = await scanImageFolder(path);
      setScanResult(result);
    } catch {
      setScanResult(null);
    }
  };

  const handleSave = async () => {
    if (!name.trim()) {
      alert("请输入任务名称");
      return;
    }
    if (!folderPath) {
      alert("请选择图片文件夹");
      return;
    }
    if (!deviceId) {
      alert("请选择目标设备");
      return;
    }

    setSaving(true);
    try {
      await onSave({
        name: name.trim(),
        folderPath,
        deviceId,
        pageId,
        intervalSeconds,
        durationType,
        endTime: durationType === "until_time" ? endTime : undefined,
        durationMinutes: durationType === "for_duration" ? durationMinutes : undefined,
      });
      onClose();
    } catch (e) {
      alert(`保存失败: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setSaving(false);
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-md p-4">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold">
            {editingTask ? "编辑循环相册任务" : "新建循环相册任务"}
          </h3>
          <button type="button" onClick={onClose} className="text-gray-500 hover:text-gray-700">
            ✕
          </button>
        </div>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">任务名称</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
              placeholder="例如: 周末旅行相册"
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">图片文件夹</label>
            <div className="flex gap-2">
              <input
                type="text"
                value={folderPath}
                onChange={(e) => setFolderPath(e.target.value)}
                className="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm"
                readOnly
              />
              <button
                type="button"
                onClick={handleSelectFolder}
                className="px-3 py-2 bg-gray-100 border border-gray-300 rounded-md text-sm hover:bg-gray-200"
              >
                选择
              </button>
            </div>
            {scanResult && (
              <p className="mt-1 text-sm text-gray-600">
                {scanResult.warning ? (
                  <span className="text-yellow-600">{scanResult.warning}</span>
                ) : (
                  `已检测到 ${scanResult.totalImages} 张图片`
                )}
              </p>
            )}
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">目标设备</label>
            <Select value={deviceId} onValueChange={setDeviceId}>
              <SelectTrigger>
                <SelectValue placeholder="请选择设备" />
              </SelectTrigger>
              <SelectContent>
                {devices.map((d) => (
                  <SelectItem key={d.deviceId} value={d.deviceId}>
                    {d.alias} ({d.deviceId.slice(0, 8)})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">目标页面</label>
            <Select value={String(pageId)} onValueChange={(v) => setPageId(Number(v))}>
              <SelectTrigger>
                <SelectValue placeholder="请选择页面" />
              </SelectTrigger>
              <SelectContent>
                {PAGE_OPTIONS.map((opt) => (
                  <SelectItem key={opt.value} value={String(opt.value)}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">循环间隔</label>
            <Select value={String(intervalSeconds)} onValueChange={(v) => setIntervalSeconds(Number(v))}>
              <SelectTrigger>
                <SelectValue placeholder="请选择间隔" />
              </SelectTrigger>
              <SelectContent>
                {INTERVAL_OPTIONS.map((opt) => (
                  <SelectItem key={opt.value} value={String(opt.value)}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">持续时间</label>
            <div className="space-y-2">
              <label className="flex items-center gap-2">
                <input
                  type="radio"
                  name="duration"
                  checked={durationType === "none"}
                  onChange={() => setDurationType("none")}
                />
                <span className="text-sm">无限制</span>
              </label>
              <label className="flex items-center gap-2">
                <input
                  type="radio"
                  name="duration"
                  checked={durationType === "until_time"}
                  onChange={() => setDurationType("until_time")}
                />
                <span className="text-sm">运行至指定时间</span>
                <input
                  type="datetime-local"
                  value={endTime}
                  onChange={(e) => setEndTime(e.target.value)}
                  disabled={durationType !== "until_time"}
                  min={getMinDateTime()}
                  className="px-2 py-1 border border-gray-300 rounded-md text-sm"
                />
              </label>
              <label className="flex items-center gap-2">
                <input
                  type="radio"
                  name="duration"
                  checked={durationType === "for_duration"}
                  onChange={() => setDurationType("for_duration")}
                />
                <span className="text-sm">运行指定时长</span>
                <input
                  type="number"
                  value={durationMinutes}
                  onChange={(e) => setDurationMinutes(Number(e.target.value))}
                  disabled={durationType !== "for_duration"}
                  className="w-16 px-2 py-1 border border-gray-300 rounded-md text-sm"
                  min={1}
                />
                <span className="text-sm">分钟</span>
              </label>
            </div>
          </div>
        </div>

        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-100"
          >
            取消
          </button>
          <button
            type="button"
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {saving ? "保存中..." : "保存任务"}
          </button>
        </div>
      </div>
    </div>
  );
}
