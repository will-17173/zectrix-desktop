import { useState, useEffect } from "react";
import { Trash2, Image, Type, PenTool } from "lucide-react";
import { getPageCacheList, deletePageCache, type PageCacheRecord } from "../../lib/tauri";

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
};

export function PageManagerPage({ devices }: Props) {
  const [selectedDeviceId, setSelectedDeviceId] = useState<string>(
    devices[0]?.deviceId || ""
  );
  const [pageList, setPageList] = useState<PageCacheRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [deletingPageId, setDeletingPageId] = useState<number | null>(null);

  useEffect(() => {
    if (!selectedDeviceId) return;
    setLoading(true);
    getPageCacheList(selectedDeviceId)
      .then(setPageList)
      .finally(() => setLoading(false));
  }, [selectedDeviceId]);

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
      <div className="h-24 bg-gray-100 rounded p-2 text-sm text-gray-600 overflow-hidden">
        {record.thumbnail || ""}
      </div>
    );
  }

  return (
    <section className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold">页面管理</h2>
        <p className="text-sm text-gray-500">
          查看和管理每个设备的墨水屏页面内容。
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
            <select
              id="device-select"
              value={selectedDeviceId}
              onChange={(e) => setSelectedDeviceId(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700"
            >
              {devices.map((d) => (
                <option key={d.deviceId} value={d.deviceId}>
                  {d.alias || d.deviceId}
                </option>
              ))}
            </select>
          </div>

          {loading && <div className="text-sm text-gray-500">加载中...</div>}

          {!loading && (
            <div className="grid grid-cols-5 gap-4">
              {PAGE_IDS.map((pageId) => {
                const record = getPageData(pageId);
                const Icon = record ? contentTypeIcon(record.contentType) : null;
                const isDeleting = deletingPageId === pageId;

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