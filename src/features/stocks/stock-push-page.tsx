import { useEffect, useState } from "react";
import { toast } from "../../components/ui/toast";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import type { StockWatchRecord } from "../../lib/tauri";

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
  watchlist: StockWatchRecord[];
  onAddStock: (code: string) => Promise<StockWatchRecord>;
  onRemoveStock: (code: string) => Promise<void>;
  onPushStocks: (deviceId: string, pageId: number) => Promise<void>;
};

function validateCode(code: string): string | null {
  if (!/^\d{6}$/.test(code)) {
    return "股票代码必须是 6 位数字";
  }

  if (!/^[036]/.test(code)) {
    return "仅支持 0、3、6 开头的 A 股代码";
  }

  return null;
}

export function StockPushPage({
  devices,
  watchlist,
  onAddStock,
  onRemoveStock,
  onPushStocks,
}: Props) {
  const [stocks, setStocks] = useState(watchlist);
  const [code, setCode] = useState("");
  const [pageId, setPageId] = useState(1);
  const [isAdding, setIsAdding] = useState(false);
  const [removingCodes, setRemovingCodes] = useState<string[]>([]);
  const [isPushing, setIsPushing] = useState(false);

  useEffect(() => {
    setStocks(watchlist);
  }, [watchlist]);

  async function handleAdd() {
    const normalizedCode = code.trim();
    const error = validateCode(normalizedCode);

    if (error) {
      toast.error(error);
      return;
    }

    if (stocks.some((stock) => stock.code === normalizedCode)) {
      toast.error(`股票代码 ${normalizedCode} 已存在`);
      return;
    }

    setIsAdding(true);
    try {
      const record = await onAddStock(normalizedCode);
      setStocks((current) => [...current, record]);
      setCode("");
      toast.success("股票已添加");
    } catch (error) {
      toast.error(`添加失败: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsAdding(false);
    }
  }

  async function handleRemove(stockCode: string) {
    setRemovingCodes((current) =>
      current.includes(stockCode) ? current : [...current, stockCode],
    );
    try {
      await onRemoveStock(stockCode);
      setStocks((current) => current.filter((stock) => stock.code !== stockCode));
      toast.success("股票已删除");
    } catch (error) {
      toast.error(`删除失败: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setRemovingCodes((current) => current.filter((code) => code !== stockCode));
    }
  }

  async function handlePush() {
    const deviceId = devices[0]?.deviceId;

    if (!deviceId) {
      toast.error("没有可用设备");
      return;
    }

    if (stocks.length === 0) {
      toast.error("请先添加股票代码");
      return;
    }

    setIsPushing(true);
    try {
      await onPushStocks(deviceId, pageId);
      toast.success(`推送成功，已发送到第 ${pageId} 页`);
    } catch (error) {
      toast.error(`推送失败: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsPushing(false);
    }
  }

  return (
    <section className="space-y-6">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold">股票推送</h2>
          <p className="text-sm text-gray-500">维护 A 股代码列表，实时获取行情后推送到设备的指定页面。</p>
        </div>
      </div>

      <div className="space-y-4 max-w-md rounded-xl border border-gray-200 bg-white/85 p-4 shadow-sm">
        <div className="space-y-2">
          <label htmlFor="stock-code" className="block text-sm font-medium">
            股票代码
          </label>
          <div className="flex gap-2">
            <input
              id="stock-code"
              value={code}
              onChange={(event) => setCode(event.target.value)}
              placeholder="例如 600519"
              inputMode="numeric"
              maxLength={6}
              className="min-w-0 flex-1 rounded-md border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
            />
            <button
              type="button"
              onClick={handleAdd}
              disabled={isAdding || !code.trim()}
              className="rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-blue-300"
            >
              {isAdding ? "添加中..." : "添加"}
            </button>
          </div>
        </div>

        <div className="space-y-2">
          <div className="text-sm font-medium">已添加股票</div>
          {stocks.length === 0 ? (
            <p className="text-sm text-gray-500">暂无股票代码</p>
          ) : (
            <ul className="divide-y divide-gray-100 overflow-hidden rounded-md border border-gray-200 bg-white">
              {stocks.map((stock) => (
                <li key={stock.code} className="flex items-center justify-between gap-3 px-3 py-2">
                  <span className="font-mono text-sm">{stock.code}</span>
                  <button
                    type="button"
                    aria-label={`删除 ${stock.code}`}
                    onClick={() => handleRemove(stock.code)}
                    disabled={removingCodes.includes(stock.code)}
                    className="text-sm text-red-600 hover:text-red-700 disabled:text-gray-400"
                  >
                    {removingCodes.includes(stock.code) ? "删除中..." : "删除"}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>

        <div className="space-y-2">
          <label id="stock-page-label" htmlFor="stock-page-trigger" className="block text-sm font-medium">
            目标页面
          </label>
          <Select value={String(pageId)} onValueChange={(value) => setPageId(Number(value))}>
            <SelectTrigger
              id="stock-page-trigger"
              aria-labelledby="stock-page-label stock-page-trigger"
            >
              <SelectValue placeholder="选择页面" />
            </SelectTrigger>
            <SelectContent>
              {PAGE_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={String(option.value)}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <button
          type="button"
          onClick={handlePush}
          disabled={isPushing}
          className="w-full rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:cursor-not-allowed disabled:bg-blue-300"
        >
          {isPushing ? "推送中..." : "推送"}
        </button>
      </div>

      <div className="text-sm text-gray-500">
        {devices.length > 0
          ? `推送到设备: ${devices[0].alias || devices[0].deviceId}`
          : "当前没有可用设备，请先在设置中添加设备。"}
      </div>
    </section>
  );
}