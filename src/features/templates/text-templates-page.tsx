import { useState } from "react";

type Device = { deviceId: string; alias: string; board: string };

const PAGE_OPTIONS = [
  { value: 1, label: "第 1 页" },
  { value: 2, label: "第 2 页" },
  { value: 3, label: "第 3 页" },
  { value: 4, label: "第 4 页" },
  { value: 5, label: "第 5 页" },
];

const FONT_SIZE_OPTIONS = [
  { value: 16, label: "16px" },
  { value: 24, label: "24px" },
  { value: 32, label: "32px" },
  { value: 48, label: "48px" },
  { value: 64, label: "64px" },
];

type Props = {
  devices: Device[];
  onPushText: (text: string, fontSize: number, deviceId: string, pageId: number) => Promise<void>;
};

export function TextTemplatesPage({ devices, onPushText }: Props) {
  const [text, setText] = useState("");
  const [fontSize, setFontSize] = useState(24);
  const [pageId, setPageId] = useState(1);
  const [pushing, setPushing] = useState(false);
  const [pushMessage, setPushMessage] = useState<string | null>(null);

  async function handlePush(e: React.FormEvent) {
    e.preventDefault();

    const deviceId = devices[0]?.deviceId;
    if (!deviceId) {
      setPushMessage("没有可用设备");
      setTimeout(() => setPushMessage(null), 3000);
      return;
    }

    if (!text.trim()) {
      setPushMessage("请输入文本内容");
      setTimeout(() => setPushMessage(null), 3000);
      return;
    }

    setPushing(true);
    try {
      await onPushText(text, fontSize, deviceId, pageId);
      setPushMessage(`推送成功，已发送到第 ${pageId} 页`);
      setText("");
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setPushMessage(`推送失败: ${errorMsg}`);
    } finally {
      setPushing(false);
      setTimeout(() => setPushMessage(null), 3000);
    }
  }

  return (
    <section className="space-y-6">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold">文本推送</h2>
          <p className="text-sm text-gray-500">直接输入文本后推送到设备的指定页面。</p>
        </div>
      </div>

      {pushMessage && (
        <div className="rounded-md bg-blue-100 px-4 py-2 text-sm text-blue-800 dark:bg-blue-900 dark:text-blue-200">
          {pushMessage}
        </div>
      )}

      <form onSubmit={handlePush} className="space-y-4 max-w-md p-4 rounded-xl border border-gray-200 bg-white/85 shadow-sm">
        <div className="space-y-2">
          <label htmlFor="text-content" className="block text-sm font-medium">文本内容</label>
          <textarea
            id="text-content"
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder="输入要推送的文本内容，支持换行"
            rows={6}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
          />
        </div>
        <div className="flex items-center gap-4">
          <div className="space-y-2">
            <label htmlFor="font-size" className="block text-sm font-medium">字体大小</label>
            <select
              id="font-size"
              value={fontSize}
              onChange={(e) => setFontSize(Number(e.target.value))}
              className="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700"
            >
              {FONT_SIZE_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
          <div className="space-y-2">
            <label htmlFor="page-id" className="block text-sm font-medium">目标页面</label>
            <select
              id="page-id"
              value={pageId}
              onChange={(e) => setPageId(Number(e.target.value))}
              className="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700"
            >
              {PAGE_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
        </div>
        <button
          type="submit"
          disabled={!text.trim() || pushing}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:cursor-not-allowed disabled:bg-blue-300"
        >
          {pushing ? "推送中..." : "推送"}
        </button>
      </form>
    </section>
  );
}