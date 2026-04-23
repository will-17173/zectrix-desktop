import { useRef, useState, useEffect, useCallback } from "react";
import { Paintbrush, Eraser } from "lucide-react";

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
  onPushSketch: (dataUrl: string, deviceId: string, pageId: number) => Promise<void>;
};

export function SketchPage({ devices, onPushSketch }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isDrawing, setIsDrawing] = useState(false);
  const [tool, setTool] = useState<"brush" | "eraser">("brush");
  const [brushSize, setBrushSize] = useState(4);
  const [brushColor, setBrushColor] = useState("#000000");
  const [selectedPageId, setSelectedPageId] = useState(1);
  const [isPushing, setIsPushing] = useState(false);
  const [pushMessage, setPushMessage] = useState<string | null>(null);

  // 初始化 canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // 设置白色背景
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, 400, 300);
  }, []);

  // 获取当前绘图上下文
  const getContext = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return null;
    const ctx = canvas.getContext("2d");
    if (!ctx) return null;
    return ctx;
  }, []);

  // 开始绘制
  const startDrawing = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const ctx = getContext();
    if (!ctx) return;

    setIsDrawing(true);
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    ctx.beginPath();
    ctx.moveTo(x, y);
  }, [getContext]);

  // 绘制
  const draw = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!isDrawing) return;
    const ctx = getContext();
    if (!ctx) return;

    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    ctx.lineWidth = brushSize;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";

    if (tool === "eraser") {
      ctx.strokeStyle = "#ffffff";
    } else {
      ctx.strokeStyle = brushColor;
    }

    ctx.lineTo(x, y);
    ctx.stroke();
  }, [isDrawing, getContext, tool, brushSize, brushColor]);

  // 结束绘制
  const stopDrawing = useCallback(() => {
    const ctx = getContext();
    if (!ctx) return;

    ctx.closePath();
    setIsDrawing(false);
  }, [getContext]);

  // 清空画布
  const clearCanvas = useCallback(() => {
    const ctx = getContext();
    if (!ctx) return;

    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, 400, 300);
  }, [getContext]);

  // 推送涂鸦
  async function handlePush() {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const deviceId = devices[0]?.deviceId;
    if (!deviceId) {
      setPushMessage("没有可用设备");
      setTimeout(() => setPushMessage(null), 3000);
      return;
    }

    setIsPushing(true);
    try {
      const dataUrl = canvas.toDataURL("image/png");
      await onPushSketch(dataUrl, deviceId, selectedPageId);
      setPushMessage(`推送成功，已发送到第 ${selectedPageId} 页`);
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
          <h2 className="text-lg font-semibold">涂鸦推送</h2>
          <p className="text-sm text-gray-500">在画布上自由绘制，然后推送到设备。</p>
        </div>
      </div>

      {pushMessage && (
        <div className="rounded-md bg-blue-100 px-4 py-2 text-sm text-blue-800 dark:bg-blue-900 dark:text-blue-200">
          {pushMessage}
        </div>
      )}

      <div className="flex gap-6">
        {/* 工具栏 */}
        <div className="flex flex-col gap-3 p-3 rounded-xl border border-gray-200 bg-white/85 shadow-sm">
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => setTool("brush")}
              className={`p-2 rounded-md ${tool === "brush" ? "bg-blue-600 text-white" : "bg-gray-200 hover:bg-gray-300"}`}
              title="画笔"
            >
              <Paintbrush size={18} />
            </button>
            <button
              type="button"
              onClick={() => setTool("eraser")}
              className={`p-2 rounded-md ${tool === "eraser" ? "bg-blue-600 text-white" : "bg-gray-200 hover:bg-gray-300"}`}
              title="橡皮擦"
            >
              <Eraser size={18} />
            </button>
          </div>

          {tool === "brush" && (
            <div className="space-y-2">
              <label className="block text-xs text-gray-500">颜色</label>
              <input
                type="color"
                value={brushColor}
                onChange={(e) => setBrushColor(e.target.value)}
                className="w-full h-8 rounded cursor-pointer"
              />
            </div>
          )}

          <div className="space-y-2">
            <label className="block text-xs text-gray-500">大小</label>
            <input
              type="range"
              min="1"
              max="20"
              value={brushSize}
              onChange={(e) => setBrushSize(Number(e.target.value))}
              className="w-full"
            />
            <span className="text-xs text-gray-500">{brushSize}px</span>
          </div>

          <button
            type="button"
            onClick={clearCanvas}
            className="px-3 py-1.5 text-sm bg-gray-200 rounded-md hover:bg-gray-300"
          >
            清空
          </button>
        </div>

        {/* Canvas */}
        <div className="flex-1">
          <canvas
            ref={canvasRef}
            width={400}
            height={300}
            className="border border-gray-300 rounded-md cursor-crosshair bg-white"
            onMouseDown={startDrawing}
            onMouseMove={draw}
            onMouseUp={stopDrawing}
            onMouseLeave={stopDrawing}
          />

          <div className="flex items-center gap-3 mt-4">
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
              onClick={handlePush}
              disabled={isPushing}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-blue-300"
            >
              {isPushing ? "推送中..." : "推送"}
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}