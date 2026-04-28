import { useEffect, useState } from "react";
import { toast } from "../../components/ui/toast";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import type { StockWatchRecord, StockPushTaskRecord, StockQuote } from "../../lib/tauri";

type Device = { deviceId: string; alias: string; board: string };

const PAGE_OPTIONS = [
  { value: 1, label: "第 1 页" },
  { value: 2, label: "第 2 页" },
  { value: 3, label: "第 3 页" },
  { value: 4, label: "第 4 页" },
  { value: 5, label: "第 5 页" },
];

const INTERVAL_OPTIONS = [
  { value: 30, label: "30 秒" },
  { value: 60, label: "1 分钟" },
  { value: 300, label: "5 分钟" },
  { value: 600, label: "10 分钟" },
];

type Props = {
  devices: Device[];
  watchlist: StockWatchRecord[];
  quotes: StockQuote[];
  pushTask: StockPushTaskRecord | null;
  onAddStock: (code: string) => Promise<StockWatchRecord>;
  onRemoveStock: (code: string) => Promise<void>;
  onPushStocks: (deviceId: string, pageId: number) => Promise<void>;
  onFetchQuotes: () => Promise<StockQuote[]>;
  onCreateTask: (deviceId: string, pageId: number, intervalSeconds: number) => Promise<StockPushTaskRecord>;
  onStartTask: () => Promise<StockPushTaskRecord>;
  onStopTask: () => Promise<StockPushTaskRecord>;
};

function validateCode(code: string): string | null {
  const trimmed = code.trim();
  if (!trimmed) return "请输入股票代码";
  const allDigits = /^\d+$/.test(trimmed);
  const hasLetter = /[a-zA-Z]/.test(trimmed);
  if (allDigits && trimmed.length > 6) return "纯数字代码不能超过 6 位";
  if (!allDigits && !hasLetter) return "无法识别的股票代码格式";
  return null;
}

function formatInterval(seconds: number): string {
  return INTERVAL_OPTIONS.find((opt) => opt.value === seconds)?.label ?? `${seconds} 秒`;
}

export function StockPushPage({
  devices,
  watchlist,
  quotes,
  pushTask,
  onAddStock,
  onRemoveStock,
  onPushStocks,
  onFetchQuotes,
  onCreateTask,
  onStartTask,
  onStopTask,
}: Props) {
  const [stocks, setStocks] = useState(watchlist);
  const [code, setCode] = useState("");
  const [pageId, setPageId] = useState(pushTask?.pageId ?? 1);
  const [intervalSeconds, setIntervalSeconds] = useState(pushTask?.intervalSeconds ?? 60);
  const [isAdding, setIsAdding] = useState(false);
  const [removingCodes, setRemovingCodes] = useState<string[]>([]);
  const [isPushing, setIsPushing] = useState(false);
  const [isLooping, setIsLooping] = useState(false);

  // 获取行情数据用于显示有效性
  useEffect(() => {
    if (watchlist.length > 0) {
      onFetchQuotes().catch(console.error);
    }
  }, [watchlist, onFetchQuotes]);

  useEffect(() => {
    setStocks(watchlist);
  }, [watchlist]);

  useEffect(() => {
    if (pushTask) {
      setPageId(pushTask.pageId);
      setIntervalSeconds(pushTask.intervalSeconds);
    }
  }, [pushTask]);

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

  async function handleStartLoop() {
    const deviceId = devices[0]?.deviceId;

    if (!deviceId) {
      toast.error("没有可用设备");
      return;
    }

    if (stocks.length === 0) {
      toast.error("请先添加股票代码");
      return;
    }

    setIsLooping(true);
    try {
      // Create task if not exists or update settings
      if (!pushTask || pushTask.pageId !== pageId || pushTask.intervalSeconds !== intervalSeconds) {
        await onCreateTask(deviceId, pageId, intervalSeconds);
      }
      await onStartTask();
      toast.success("循环推送已启动");
    } catch (error) {
      toast.error(`启动失败: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsLooping(false);
    }
  }

  async function handleStopLoop() {
    setIsLooping(true);
    try {
      await onStopTask();
      toast.success("循环推送已停止");
    } catch (error) {
      toast.error(`停止失败: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsLooping(false);
    }
  }

  const isRunning = pushTask?.status === "running";

  // 根据 quotes 数据判断股票有效性
  const getQuoteForCode = (code: string) => quotes.find((q) => q.code === code);

  return (
    <section className="space-y-6">
      <header className="rounded-lg bg-gradient-to-r from-red-50 to-rose-50 px-4 py-3 border border-red-100">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">股票推送</h2>
            <p className="text-sm text-gray-500">维护股票代码列表（A股/港股/美股），实时获取行情后推送到设备的指定页面。</p>
          </div>
        </div>
      </header>

      <div className="space-y-4 max-w-md rounded-xl border border-red-200 bg-gradient-to-br from-red-50/30 to-white p-4 shadow-sm">
        <div className="space-y-2">
          <label htmlFor="stock-code" className="block text-sm font-medium text-gray-700">
            股票代码
          </label>
          <div className="flex gap-2">
            <input
              id="stock-code"
              value={code}
              onChange={(event) => setCode(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !isAdding && code.trim()) {
                  event.preventDefault();
                  void handleAdd();
                }
              }}
              placeholder="A股: 600519 | 港股: 00700 | 美股: AAPL"
              maxLength={10}
              className="min-w-0 flex-1 rounded-md border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-red-500"
            />
            <button
              type="button"
              onClick={handleAdd}
              disabled={isAdding || !code.trim()}
              className="rounded-md bg-red-500 px-4 py-2 text-white hover:bg-red-600 disabled:cursor-not-allowed disabled:bg-red-300 shadow-sm transition"
            >
              {isAdding ? "添加中..." : "添加"}
            </button>
          </div>
        </div>

        <div className="space-y-2">
          <div className="text-sm font-medium text-gray-700">已添加股票</div>
          {stocks.length === 0 ? (
            <div className="rounded-lg border border-dashed border-red-300 bg-red-50/30 px-4 py-4 text-sm text-red-500">
              暂无股票代码
            </div>
          ) : (
            <ul className="divide-y divide-gray-100 overflow-hidden rounded-md border border-red-200 bg-white">
              {stocks.map((stock) => {
                const quote = getQuoteForCode(stock.code);
                const isInvalid = quote ? !quote.valid : false;
                const stockName = quote?.name;
                return (
                  <li
                    key={stock.code}
                    className={`flex items-center justify-between gap-3 px-3 py-2 ${isInvalid ? "bg-gray-100 text-gray-400" : ""}`}
                  >
                    <div className="flex items-center gap-2">
                      <div className={`w-1.5 h-1.5 rounded-full ${isInvalid ? "bg-gray-300" : "bg-red-400"}`}></div>
                      <span className="font-mono text-sm flex items-center gap-1.5">
                        {stock.code}
                        {stock.market === "hk" && (
                          <span className="rounded px-1 py-0.5 text-xs bg-blue-100 text-blue-700">港</span>
                        )}
                        {stock.market === "us" && (
                          <span className="rounded px-1 py-0.5 text-xs bg-green-100 text-green-700">美</span>
                        )}
                        {stockName ? <span className="text-xs text-gray-500">({stockName})</span> : null}
                      </span>
                    </div>
                    <button
                      type="button"
                      aria-label={`删除 ${stock.code}`}
                      onClick={() => handleRemove(stock.code)}
                      disabled={removingCodes.includes(stock.code)}
                      className="text-sm text-red-600 hover:text-red-700 disabled:text-gray-400 transition"
                    >
                      {removingCodes.includes(stock.code) ? "删除中..." : "删除"}
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
        </div>

        <div className="space-y-2">
          <label id="stock-page-label" htmlFor="stock-page-trigger" className="block text-sm font-medium text-gray-700">
            目标页面
          </label>
          <Select value={String(pageId)} onValueChange={(value) => setPageId(Number(value))} disabled={isRunning}>
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

        <div className="space-y-2">
          <label id="stock-interval-label" htmlFor="stock-interval-trigger" className="block text-sm font-medium text-gray-700">
            推送间隔
          </label>
          <Select value={String(intervalSeconds)} onValueChange={(value) => setIntervalSeconds(Number(value))} disabled={isRunning}>
            <SelectTrigger
              id="stock-interval-trigger"
              aria-labelledby="stock-interval-label stock-interval-trigger"
            >
              <SelectValue placeholder="选择间隔" />
            </SelectTrigger>
            <SelectContent>
              {INTERVAL_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={String(option.value)}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="flex gap-2">
          <button
            type="button"
            onClick={handlePush}
            disabled={isPushing || isRunning}
            className="flex-1 rounded-md bg-red-500 px-4 py-2 text-white hover:bg-red-600 focus:outline-none focus:ring-2 focus:ring-red-500 disabled:cursor-not-allowed disabled:bg-red-300 shadow-sm transition"
          >
            {isPushing ? "推送中..." : "单次推送"}
          </button>
          {!isRunning ? (
            <button
              type="button"
              onClick={handleStartLoop}
              disabled={isLooping}
              className="flex-1 rounded-md bg-rose-500 px-4 py-2 text-white hover:bg-rose-600 focus:outline-none focus:ring-2 focus:ring-rose-500 disabled:cursor-not-allowed disabled:bg-rose-300 shadow-sm transition"
            >
              {isLooping ? "启动中..." : "开始循环"}
            </button>
          ) : (
            <button
              type="button"
              onClick={handleStopLoop}
              disabled={isLooping}
              className="flex-1 rounded-md bg-red-500 px-4 py-2 text-white hover:bg-red-600 focus:outline-none focus:ring-2 focus:ring-red-500 disabled:cursor-not-allowed disabled:bg-red-300 shadow-sm transition"
            >
              {isLooping ? "停止中..." : "停止循环"}
            </button>
          )}
        </div>
      </div>

      {pushTask && (
        <div className="max-w-md rounded-xl border border-rose-200 bg-gradient-to-br from-rose-50/30 to-white p-4 shadow-sm">
          <div className="flex items-center gap-2 mb-3">
            <div className="w-1.5 h-1.5 rounded-full bg-rose-400"></div>
            <div className="text-sm font-medium text-gray-900">任务状态</div>
          </div>
          <div className="text-sm text-gray-600 space-y-1">
            <p>
              状态: <span className={`font-medium ${isRunning ? "text-red-600" : "text-gray-500"}`}>{isRunning ? "运行中" : "已停止"}</span>
            </p>
            <p>
              设备: {devices.find((d) => d.deviceId === pushTask.deviceId)?.alias ?? pushTask.deviceId}
            </p>
            <p>页面: 第 {pushTask.pageId} 页</p>
            <p>间隔: {formatInterval(pushTask.intervalSeconds)}</p>
            {pushTask.lastPushAt && (
              <p>上次推送: {new Date(pushTask.lastPushAt).toLocaleString()}</p>
            )}
            {pushTask.errorMessage && (
              <p className="text-red-600">错误: {pushTask.errorMessage}</p>
            )}
          </div>
        </div>
      )}

      <div className="text-sm text-gray-500">
        {devices.length > 0
          ? `推送到设备: ${devices[0].alias || devices[0].deviceId}`
          : "当前没有可用设备，请先在设置中添加设备。"}
      </div>
    </section>
  );
}
