import { useState } from "react";

type Device = { deviceId: string; alias: string; board: string };

export type TextTemplateRecord = {
  id: number;
  title: string;
  content: string;
};

const PAGE_OPTIONS = [
  { value: 1, label: "第 1 页" },
  { value: 2, label: "第 2 页" },
  { value: 3, label: "第 3 页" },
  { value: 4, label: "第 4 页" },
  { value: 5, label: "第 5 页" },
];

type Props = {
  templates: TextTemplateRecord[];
  devices: Device[];
  onCreateTemplate: (input: { title: string; content: string }) => Promise<TextTemplateRecord>;
  onPushTemplate: (templateId: number, deviceId: string, pageId?: number) => Promise<void>;
};

export function TextTemplatesPage({ templates: initialTemplates, devices, onCreateTemplate, onPushTemplate }: Props) {
  const [templates, setTemplates] = useState<TextTemplateRecord[]>(initialTemplates);
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [pageIds, setPageIds] = useState<Record<number, number>>({});
  const [pushingId, setPushingId] = useState<number | null>(null);
  const [pushMessage, setPushMessage] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const created = await onCreateTemplate({ title, content });
    setTemplates((prev) => [...prev, created]);
    setTitle("");
    setContent("");
  }

  async function handlePush(templateId: number) {
    const deviceId = devices[0]?.deviceId;
    if (!deviceId) {
      setPushMessage("没有可用设备");
      setTimeout(() => setPushMessage(null), 3000);
      return;
    }

    const pageId = pageIds[templateId] ?? 1;
    setPushingId(templateId);
    try {
      await onPushTemplate(templateId, deviceId, pageId);
      setPushMessage(`推送成功，已发送到第 ${pageId} 页`);
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
          <h2 className="text-lg font-semibold">文本模板</h2>
          <p className="text-sm text-gray-500">创建文本模板后可推送到设备的指定页面。</p>
        </div>
      </div>

      {pushMessage && (
        <div className="rounded-md bg-blue-100 px-4 py-2 text-sm text-blue-800 dark:bg-blue-900 dark:text-blue-200">
          {pushMessage}
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-4 max-w-md p-4 rounded-xl border border-gray-200 bg-white/85 shadow-sm">
        <div className="space-y-2">
          <label htmlFor="template-title" className="block text-sm font-medium">标题</label>
          <input
            id="template-title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="可选，最多200字"
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
          />
        </div>
        <div className="space-y-2">
          <label htmlFor="template-content" className="block text-sm font-medium">正文</label>
          <textarea
            id="template-content"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="最多5000字，支持换行"
            rows={4}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
          />
        </div>
        <button
          type="submit"
          disabled={!title && !content}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:cursor-not-allowed disabled:bg-blue-300"
        >
          保存模板
        </button>
      </form>

      <ul className="grid gap-3">
        {templates.map((t) => (
          <li key={t.id} className="flex items-center gap-3 p-3 rounded-xl border border-gray-200 bg-white/85 shadow-sm dark:border-gray-700">
            <div className="flex-1 min-w-0">
              <p className="font-medium truncate">{t.title || "无标题"}</p>
              <p className="text-sm text-gray-500 truncate">{t.content || "无内容"}</p>
            </div>
            <select
              value={pageIds[t.id] ?? 1}
              onChange={(e) => setPageIds((prev) => ({ ...prev, [t.id]: Number(e.target.value) }))}
              className="px-3 py-1.5 border border-gray-300 rounded-md text-sm dark:border-gray-600 dark:bg-gray-700"
            >
              {PAGE_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
            <button
              type="button"
              onClick={() => handlePush(t.id)}
              disabled={pushingId === t.id}
              className="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-blue-300"
            >
              {pushingId === t.id ? "推送中..." : "推送"}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}