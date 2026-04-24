import { useState } from "react";
import { toast } from "../../components/ui/toast";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";

type Device = { deviceId: string; alias: string; board: string };

const PAGE_OPTIONS = [
  { value: 1, label: "第 1 页" },
  { value: 2, label: "第 2 页" },
  { value: 3, label: "第 3 页" },
  { value: 4, label: "第 4 页" },
  { value: 5, label: "第 5 页" },
];

type Props = {
  devices: Device[];
  onPushText: (title: string, body: string, deviceId: string, pageId: number) => Promise<void>;
};

export function TextTemplatesPage({ devices, onPushText }: Props) {
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [pageId, setPageId] = useState(1);
  const [pushing, setPushing] = useState(false);

  async function handlePush(e: React.FormEvent) {
    e.preventDefault();

    const deviceId = devices[0]?.deviceId;
    if (!deviceId) {
      toast.error("没有可用设备");
      return;
    }

    if (!title.trim() && !body.trim()) {
      toast.error("请输入标题或正文");
      return;
    }

    setPushing(true);
    try {
      await onPushText(title, body, deviceId, pageId);
      toast.success(`推送成功，已发送到第 ${pageId} 页`);
      setTitle("");
      setBody("");
    } catch (e) {
      toast.error(`推送失败: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setPushing(false);
    }
  }

  return (
    <section className="space-y-6">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold">文本推送</h2>
          <p className="text-sm text-gray-500">输入标题和正文后推送到设备的指定页面。</p>
        </div>
      </div>

      <form onSubmit={handlePush} className="space-y-4 max-w-md p-4 rounded-xl border border-gray-200 bg-white/85 shadow-sm">
        <div className="space-y-2">
          <label htmlFor="text-title" className="block text-sm font-medium">标题</label>
          <input
            id="text-title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="输入标题，最多 200 字"
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
          />
        </div>
        <div className="space-y-2">
          <label htmlFor="text-body" className="block text-sm font-medium">正文</label>
          <textarea
            id="text-body"
            value={body}
            onChange={(e) => setBody(e.target.value)}
            placeholder="输入正文内容，支持换行"
            rows={6}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
          />
        </div>
        <div className="flex items-center gap-4">
          <div className="space-y-2">
            <label htmlFor="page-id" className="block text-sm font-medium">目标页面</label>
            <Select value={String(pageId)} onValueChange={(v) => setPageId(Number(v))}>
              <SelectTrigger id="page-id">
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
        </div>
        <button
          type="submit"
          disabled={(!title.trim() && !body.trim()) || pushing}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:cursor-not-allowed disabled:bg-blue-300"
        >
          {pushing ? "推送中..." : "推送"}
        </button>
      </form>
    </section>
  );
}
