import { useState, useEffect } from "react";
import { Trash2, Image, Type, PenTool, Repeat, Pause } from "lucide-react";
import { getPageCacheList, deletePageCache, type PageCacheRecord } from "../../lib/tauri";
import { listImageLoopTasks, startImageLoopTask, stopImageLoopTask, type ImageLoopTask } from "../../lib/tauri";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";

type Device = { deviceId: string; alias: string; board: string };

const PAGE_IDS = [1, 2, 3, 4, 5];

function contentTypeIcon(type: string) {
  switch (type) {
    case "sketch":
      return PenTool;
    case "image":
      return Image;
    case "text":
    case "structured_text":
      return Type;
    default:
      return Type;
  }
}

function contentTypeLabel(type: string) {
  switch (type) {
    case "sketch":
      return "涂鸦";
    case "image":
      return "图片";
    case "text":
      return "文本";
    case "structured_text":
      return "结构化文本";
    default:
      return "未知";
  }
}

type Props = {
  devices: Device[];
  onRefreshLoopTasks?: () => Promise<void>;
};

export function PageManagerPage({ devices, onRefreshLoopTasks }: Props) {
  const [selectedDeviceId, setSelectedDeviceId] = useState<string>(
    devices[0]?.deviceId || ""
  );
  const [pageList, setPageList] = useState<PageCacheRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [deletingPageId, setDeletingPageId] = useState<number | null>(null);
  const [loopTasks, setLoopTasks] = useState<ImageLoopTask[]>([]);
  const [togglingTaskId, setTogglingTaskId] = useState<number | null>(null);

  useEffect(() => {
    if (!selectedDeviceId) return;
    setLoading(true);
    getPageCacheList(selectedDeviceId)
      .then(setPageList)
      .finally(() => setLoading(false));
  }, [selectedDeviceId]);

  useEffect(() => {
    listImageLoopTasks()
      .then(setLoopTasks);
  }, []);

  async function handleDelete(pageId: number) {
    setDeletingPageId(pageId);
    try {
      await deletePageCache(selectedDeviceId, pageId);
      const updated = await getPageCacheList(selectedDeviceId);
      setPageList(updated);
    } catch (e) {
      console.error("删除页面失败:", e);
    } finally {
      setDeletingPageId(null);
    }
  }

  function getPageData(pageId: number): PageCacheRecord | undefined {
    return pageList.find((p) => p.pageId === pageId);
  }

  function getPageLoopTask(pageId: number): ImageLoopTask | undefined {
    return loopTasks.find(
      (t) => t.deviceId === selectedDeviceId && t.pageId === pageId && t.status === "running"
    );
  }

  async function handleToggleTask(task: ImageLoopTask) {
    setTogglingTaskId(task.id);
    try {
      const updated = task.status === "running"
        ? await stopImageLoopTask(task.id)
        : await startImageLoopTask(task.id);
      setLoopTasks((prev) => prev.map((t) => (t.id === updated.id ? updated : t)));
      await onRefreshLoopTasks?.();
    } catch (e) {
      console.error("切换任务失败:", e);
    } finally {
      setTogglingTaskId(null);
    }
  }

  function renderThumbnail(record: PageCacheRecord) {
    if (record.contentType === "sketch" || record.contentType === "image") {
      const Icon = contentTypeIcon(record.contentType);
      return (
        <div className="flex items-center justify-center h-24 bg-gray-100 rounded">
          <Icon size={32} className="text-gray-400" />
        </div>
      );
    }
    return (
      <div className="h-24 bg-gray-100 rounded p-2 text-sm text-gray-600 overflow-hidden whitespace-pre-wrap">
        {record.thumbnail || ""}
      </div>
    );
  }

  return (
    <section className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold">页面管理</h2>
        <p className="text-sm text-gray-500">
          注意：你在其它设备推送的页面内容不会同步到这里。
        </p>
      </div>

      {devices.length === 0 && (
        <div className="text-sm text-gray-500">请先在设置中添加设备。</div>
      )}

      {devices.length > 0 && (
        <div className="space-y-4">
          <div className="space-y-2">
            <label htmlFor="device-select" className="block text-sm font-medium">
              选择设备
            </label>
            <Select value={selectedDeviceId} onValueChange={setSelectedDeviceId}>
              <SelectTrigger id="device-select">
                <SelectValue placeholder="选择设备" />
              </SelectTrigger>
              <SelectContent>
                {devices.map((d) => (
                  <SelectItem key={d.deviceId} value={d.deviceId}>
                    {d.alias || d.deviceId}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {loading && <div className="text-sm text-gray-500">加载中...</div>}

          {!loading && (
            <div className="grid grid-cols-5 gap-4">
              {PAGE_IDS.map((pageId) => {
                const record = getPageData(pageId);
                const Icon = record ? contentTypeIcon(record.contentType) : null;
                const isDeleting = deletingPageId === pageId;
                const loopTask = getPageLoopTask(pageId);

                return (
                  <div
                    key={pageId}
                    className="p-4 rounded-xl border border-gray-200 bg-white/85 shadow-sm space-y-2"
                  >
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium">第 {pageId} 页</span>
                      {Icon && (
                        <Icon size={16} className="text-gray-400" />
                      )}
                    </div>

                    {record ? (
                      <>
                        {renderThumbnail(record)}
                        <div className="text-xs text-gray-500">
                          {contentTypeLabel(record.contentType)}
                        </div>
                        <button
                          type="button"
                          onClick={() => handleDelete(pageId)}
                          disabled={isDeleting}
                          className="w-full flex items-center justify-center gap-1 px-2 py-1 text-sm text-red-600 hover:bg-red-50 rounded disabled:opacity-50"
                        >
                          <Trash2 size={14} />
                          {isDeleting ? "删除中..." : "删除"}
                        </button>
                      </>
                    ) : loopTask ? (
                      <div className="flex flex-col items-center justify-center h-32 gap-1">
                        <div className="flex items-center gap-1 text-sm text-emerald-600 font-medium">
                          <Repeat size={14} />
                          <span>{loopTask.name}</span>
                        </div>
                        <button
                          type="button"
                          onClick={() => handleToggleTask(loopTask)}
                          disabled={togglingTaskId === loopTask.id}
                          className="flex items-center gap-0.5 px-2 py-0.5 text-xs text-red-500 hover:bg-red-50 rounded disabled:opacity-50"
                        >
                          <Pause size={10} />
                          {togglingTaskId === loopTask.id ? "停止中" : "停止"}
                        </button>
                      </div>
                    ) : (
                      <div className="flex items-center justify-center h-32 text-sm text-gray-400">
                        暂无内容
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}
    </section>
  );
}