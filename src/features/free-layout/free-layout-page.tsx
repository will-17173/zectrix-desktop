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
  { value: 12, label: "12px" },
  { value: 16, label: "16px" },
  { value: 20, label: "20px (默认)" },
  { value: 24, label: "24px" },
  { value: 32, label: "32px" },
  { value: 48, label: "48px" },
];

type Props = {
  devices: Device[];
  onPushText: (text: string, fontSize: number, deviceId: string, pageId: number) => Promise<void>;
};

export function FreeLayoutPage({ devices, onPushText }: Props) {
  const [text, setText] = useState("");
  const [fontSize, setFontSize] = useState(20);
  const [pageId, setPageId] = useState(1);
  const [isPushing, setIsPushing] = useState(false);
  const [pushMessage, setPushMessage] = useState<string | null>(null);

  async function handlePush() {
    if (!text.trim()) {
      setPushMessage("请输入文本内容");
      setTimeout(() => setPushMessage(null), 3000);
      return;
    }

    const deviceId = devices[0]?.deviceId;
    if (!deviceId) {
      setPushMessage("没有可用设备");
      setTimeout(() => setPushMessage(null), 3000);
      return;
    }

    setIsPushing(true);
    try {
      await onPushText(text, fontSize, deviceId, pageId);
      setPushMessage(`推送成功，已发送到第 ${pageId} 页`);
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setPushMessage(`推送失败: ${errorMsg}`);
    } finally {
      setIsPushing(false);
      setTimeout(() => setPushMessage(null), 3000);
    }
  }

  return (
    <section className="space-y-6">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold">自由排版</h2>
          <p className="text-sm text-gray-500">输入文本内容，设置字号，推送到设备的指定页面。</p>
        </div>
      </div>

      {pushMessage && (
        <div className="rounded-md bg-blue-100 px-4 py-2 text-sm text-blue-800 dark:bg-blue-900 dark:text-blue-200">
          {pushMessage}
        </div>
      )}

      <div className="space-y-4 max-w-md p-4 rounded-xl border border-gray-200 bg-white/85 shadow-sm">
        <div className="space-y-2">
          <label htmlFor="text-content" className="block text-sm font-medium">文本内容</label>
          <textarea
            id="text-content"
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder="最多5000字，支持换行"
            rows={6}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
          />
        </div>

        <div className="space-y-2">
          <label htmlFor="font-size" className="block text-sm font-medium">字号</label>
          <select
            id="font-size"
            value={fontSize}
            onChange={(e) => setFontSize(Number(e.target.value))}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
          >
            {FONT_SIZE_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>

        <div className="space-y-2">
          <label htmlFor="page-id" className="block text-sm font-medium">页码</label>
          <select
            id="page-id"
            value={pageId}
            onChange={(e) => setPageId(Number(e.target.value))}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
          >
            {PAGE_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>

        <button
          type="button"
          onClick={handlePush}
          disabled={isPushing || !text.trim()}
          className="w-full px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:cursor-not-allowed disabled:bg-blue-300"
        >
          {isPushing ? "推送中..." : "推送"}
        </button>
      </div>

      {devices.length > 0 && (
        <div className="text-sm text-gray-500">
          推送到设备: {devices[0].alias || devices[0].deviceId}
        </div>
      )}
    </section>
  );
}