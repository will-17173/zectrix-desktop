import { useState } from "react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import { toast } from "../../components/ui/toast";

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

  async function handlePush() {
    if (!text.trim()) {
      toast.error("请输入文本内容");
      return;
    }

    const deviceId = devices[0]?.deviceId;
    if (!deviceId) {
      toast.error("没有可用设备");
      return;
    }

    setIsPushing(true);
    try {
      await onPushText(text, fontSize, deviceId, pageId);
      toast.success(`推送成功，已发送到第 ${pageId} 页`);
    } catch (e) {
      toast.error(`推送失败: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setIsPushing(false);
    }
  }

  return (
    <section className="space-y-6">
      <header className="rounded-lg bg-gradient-to-r from-amber-50 to-orange-50 px-4 py-3 border border-amber-100">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">自由排版</h2>
            <p className="text-sm text-gray-500">输入文本内容，设置字号，推送到设备的指定页面。</p>
          </div>
        </div>
      </header>

      <div className="space-y-4 max-w-md p-4 rounded-xl border border-amber-200 bg-gradient-to-br from-amber-50/30 to-white shadow-sm">
        <div className="space-y-2">
          <label htmlFor="text-content" className="block text-sm font-medium text-gray-700">文本内容</label>
          <textarea
            id="text-content"
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder="最多5000字，支持换行"
            rows={6}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-amber-500"
          />
        </div>

        <div className="space-y-2">
          <label id="font-size-label" htmlFor="font-size-trigger" className="block text-sm font-medium text-gray-700">字号</label>
          <Select value={String(fontSize)} onValueChange={(value) => setFontSize(Number(value))}>
            <SelectTrigger
              id="font-size-trigger"
              aria-labelledby="font-size-label font-size-trigger"
            >
              <SelectValue placeholder="选择字号" />
            </SelectTrigger>
            <SelectContent>
              {FONT_SIZE_OPTIONS.map((opt) => (
                <SelectItem key={opt.value} value={String(opt.value)}>
                  {opt.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <label id="page-id-label" htmlFor="page-id-trigger" className="block text-sm font-medium text-gray-700">页码</label>
          <Select value={String(pageId)} onValueChange={(value) => setPageId(Number(value))}>
            <SelectTrigger
              id="page-id-trigger"
              aria-labelledby="page-id-label page-id-trigger"
            >
              <SelectValue placeholder="选择页面" />
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

        <button
          type="button"
          onClick={handlePush}
          disabled={isPushing || !text.trim()}
          className="w-full px-4 py-2 bg-amber-500 text-white rounded-md hover:bg-amber-600 focus:outline-none focus:ring-2 focus:ring-amber-500 disabled:cursor-not-allowed disabled:bg-amber-300 shadow-sm transition"
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