import { ChangeEvent, useMemo, useState } from "react";

type Props = {
  onSave: (input: {
    name: string;
    sourcePath?: string;
    sourceDataUrl?: string;
    crop: { x: number; y: number; width: number; height: number };
    rotation: number;
    flipX: boolean;
    flipY: boolean;
  }) => Promise<void>;
  onClose: () => void;
};

export function ImageEditorDialog({ onSave, onClose }: Props) {
  const [name, setName] = useState("");
  const [sourceDataUrl, setSourceDataUrl] = useState<string | undefined>();
  const [selectedFileName, setSelectedFileName] = useState("");

  const canSave = useMemo(() => name.trim() && sourceDataUrl, [name, sourceDataUrl]);

  function handleFileChange(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) {
      setSourceDataUrl(undefined);
      setSelectedFileName("");
      return;
    }

    const reader = new FileReader();
    reader.onload = () => {
      const result = typeof reader.result === "string" ? reader.result : undefined;
      setSourceDataUrl(result);
      setSelectedFileName(file.name);
      if (!name.trim()) {
        setName(file.name.replace(/\.[^.]+$/, ""));
      }
    };
    reader.readAsDataURL(file);
  }

  async function handleConfirm() {
    if (!canSave) {
      return;
    }
    await onSave({
      name: name.trim(),
      sourceDataUrl,
      crop: { x: 0, y: 0, width: 400, height: 300 },
      rotation: 0,
      flipX: false,
      flipY: false,
    });
  }

  return (
    <div
      role="dialog"
      aria-label="图片编辑器"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
    >
      <div className="w-full max-w-2xl rounded-2xl bg-white p-6 shadow-lg dark:bg-gray-800">
        <div className="mb-4 flex items-start justify-between gap-4">
          <div>
            <h2 className="text-xl font-semibold">400x300 效果预览</h2>
            <p className="text-sm text-gray-500">导入图片后会保存在应用本地图库，只有手动推送时才会发送到设备。</p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-md px-3 py-1 text-sm text-gray-600 hover:bg-gray-100"
          >
            关闭
          </button>
        </div>

        <div className="grid gap-6 md:grid-cols-[400px_minmax(0,1fr)]">
          <div className="overflow-hidden rounded-xl border border-gray-200 bg-gray-100">
            {sourceDataUrl ? (
              <img src={sourceDataUrl} alt="导入预览" className="h-[300px] w-full object-cover" />
            ) : (
              <div className="flex h-[300px] items-center justify-center text-sm text-gray-500">请选择一张图片</div>
            )}
          </div>

          <div className="space-y-4">
            <div className="space-y-2">
              <label htmlFor="image-source" className="block text-sm font-medium">选择图片</label>
              <input
                id="image-source"
                type="file"
                accept="image/*"
                onChange={handleFileChange}
                className="hidden"
              />
              <button
                type="button"
                onClick={() => document.getElementById("image-source")?.click()}
                className="px-4 py-2 bg-gray-200 text-sm rounded-md hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600"
              >
                {selectedFileName ? "更换图片" : "选择图片"}
              </button>
              {selectedFileName ? <p className="text-sm text-gray-500">已选择：{selectedFileName}</p> : null}
            </div>

            <div className="space-y-2">
              <label htmlFor="image-name" className="block text-sm font-medium">图片名称</label>
              <input
                id="image-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600"
              />
            </div>

            <div className="flex gap-2 pt-2">
              <button
                type="button"
                onClick={handleConfirm}
                disabled={!canSave}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-blue-300"
              >
                保存到图库
              </button>
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 bg-gray-200 rounded-md hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600"
              >
                取消
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
