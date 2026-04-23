import { useEffect, useState } from "react";
import { ImageEditorDialog } from "./image-editor-dialog";

type Device = { deviceId: string; alias: string; board: string };

export type ImageTemplateRecord = {
  id: number;
  name: string;
  filePath: string;
};

type ImageTemplateWithThumbnail = ImageTemplateRecord & {
  thumbnail?: string;
  thumbnailLoading?: boolean;
};

type Props = {
  templates: ImageTemplateRecord[];
  devices: Device[];
  onSaveTemplate: (input: {
    name: string;
    sourcePath?: string;
    sourceDataUrl?: string;
    crop: { x: number; y: number; width: number; height: number };
    rotation: number;
    flipX: boolean;
    flipY: boolean;
  }) => Promise<ImageTemplateRecord>;
  onPushTemplate: (templateId: number, deviceId: string, pageId: number) => Promise<void>;
  onLoadThumbnail?: (templateId: number) => Promise<string>;
};

const PAGE_OPTIONS = [
  { value: 1, label: "第 1 页" },
  { value: 2, label: "第 2 页" },
  { value: 3, label: "第 3 页" },
  { value: 4, label: "第 4 页" },
  { value: 5, label: "第 5 页" },
];

export function ImageTemplatesPage({
  templates: initialTemplates,
  devices,
  onSaveTemplate,
  onPushTemplate,
  onLoadThumbnail,
}: Props) {
  const [templates, setTemplates] = useState<ImageTemplateWithThumbnail[]>([]);
  const [editorOpen, setEditorOpen] = useState(false);
  const [pushingId, setPushingId] = useState<number | null>(null);
  const [pushMessage, setPushMessage] = useState<string | null>(null);
  const [selectedPageId, setSelectedPageId] = useState<number>(1);

  // 初始化并加载缩略图
  useEffect(() => {
    setTemplates(
      initialTemplates.map((t) => ({
        ...t,
        thumbnail: undefined,
        thumbnailLoading: true,
      }))
    );
  }, [initialTemplates]);

  // 加载每个模板的缩略图
  useEffect(() => {
    if (!onLoadThumbnail) return;
    for (const t of templates) {
      if (t.thumbnailLoading && !t.thumbnail) {
        onLoadThumbnail(t.id)
          .then((thumbnail) => {
            setTemplates((prev) =>
              prev.map((item) =>
                item.id === t.id
                  ? { ...item, thumbnail, thumbnailLoading: false }
                  : item
              )
            );
          })
          .catch(() => {
            setTemplates((prev) =>
              prev.map((item) =>
                item.id === t.id ? { ...item, thumbnailLoading: false } : item
              )
            );
          });
      }
    }
  }, [templates, onLoadThumbnail]);

  async function handleSave(input: {
    name: string;
    sourcePath?: string;
    sourceDataUrl?: string;
    crop: { x: number; y: number; width: number; height: number };
    rotation: number;
    flipX: boolean;
    flipY: boolean;
  }) {
    const created = await onSaveTemplate(input);
    setTemplates((prev) => [
      { ...created, thumbnail: undefined, thumbnailLoading: true },
      ...prev,
    ]);
    setEditorOpen(false);
  }

  async function handlePush(templateId: number) {
    const deviceId = devices[0]?.deviceId;
    if (!deviceId) {
      setPushMessage("没有可用设备");
      setTimeout(() => setPushMessage(null), 3000);
      return;
    }

    setPushingId(templateId);
    try {
      await onPushTemplate(templateId, deviceId, selectedPageId);
      setPushMessage(`推送成功，已发送到第 ${selectedPageId} 页`);
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setPushMessage(`推送失败: ${errorMsg}`);
    } finally {
      setPushingId(null);
      setTimeout(() => setPushMessage(null), 3000);
    }
  }

  return (
    <section className="space-y-6">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold">本地图库</h2>
          <p className="text-sm text-gray-500">导入后的图片会先保存在应用本地，选中后再单独推送到设备。</p>
        </div>
        <div className="flex items-center gap-3">
          <select
            value={selectedPageId}
            onChange={(e) => setSelectedPageId(Number(e.target.value))}
            className="px-3 py-2 border border-gray-300 rounded-md text-sm dark:border-gray-600 dark:bg-gray-700"
          >
            {PAGE_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
          <button
            type="button"
            onClick={() => setEditorOpen(true)}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            导入图片
          </button>
        </div>
      </div>

      {pushMessage && (
        <div className="rounded-md bg-blue-100 px-4 py-2 text-sm text-blue-800 dark:bg-blue-900 dark:text-blue-200">
          {pushMessage}
        </div>
      )}

      {editorOpen && (
        <ImageEditorDialog onSave={handleSave} onClose={() => setEditorOpen(false)} />
      )}

      <ul className="grid gap-3 grid-cols-5">
        {templates.map((t) => (
          <li key={t.id} className="rounded-xl border border-gray-200 bg-white/85 shadow-sm overflow-hidden">
            <div className="relative">
              <div className="w-full h-[75px] bg-gray-100 flex items-center justify-center">
                {t.thumbnail ? (
                  <img
                    src={`data:image/png;base64,${t.thumbnail}`}
                    alt={t.name}
                    className="w-full h-full object-cover"
                  />
                ) : t.thumbnailLoading ? (
                  <span className="text-xs text-gray-400">加载中...</span>
                ) : (
                  <span className="text-xs text-gray-400">无预览</span>
                )}
              </div>
              <button
                type="button"
                onClick={() => handlePush(t.id)}
                disabled={pushingId === t.id}
                className="absolute inset-0 flex items-center justify-center bg-black/50 text-white text-sm opacity-0 hover:opacity-100 transition-opacity disabled:opacity-100 disabled:bg-black/30"
              >
                {pushingId === t.id ? "推送中..." : "推送"}
              </button>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
