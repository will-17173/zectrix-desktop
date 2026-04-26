import { useEffect, useState, useCallback } from "react";
import { confirm } from "@tauri-apps/plugin-dialog";
import { ImageEditorDialog } from "./image-editor-dialog";
import { ImageLoopTaskList } from "./image-loop-task-list";
import { ImageLoopTaskDialog } from "./image-loop-task-dialog";
import { useImageLoopRunner } from "./use-image-loop-runner";
import { type ImageLoopTask, type ImageLoopTaskInput, type DeviceRecord } from "../../lib/tauri";
import { toast } from "../../components/ui/toast";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";

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
  devices: DeviceRecord[];
  imageLoopTasks: ImageLoopTask[];
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
  onDeleteTemplate: (templateId: number) => Promise<void>;
  onLoadThumbnail?: (templateId: number) => Promise<string>;
  onCreateLoopTask: (input: ImageLoopTaskInput) => Promise<void>;
  onUpdateLoopTask: (taskId: number, input: ImageLoopTaskInput) => Promise<void>;
  onDeleteLoopTask: (taskId: number) => Promise<void>;
  onStartLoopTask: (taskId: number) => Promise<ImageLoopTask>;
  onStopLoopTask: (taskId: number) => Promise<ImageLoopTask>;
  onRefreshLoopTasks: () => Promise<void>;
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
  imageLoopTasks,
  onSaveTemplate,
  onPushTemplate,
  onDeleteTemplate,
  onLoadThumbnail,
  onCreateLoopTask,
  onUpdateLoopTask,
  onDeleteLoopTask,
  onStartLoopTask,
  onStopLoopTask,
  onRefreshLoopTasks,
}: Props) {
  const [templates, setTemplates] = useState<ImageTemplateWithThumbnail[]>([]);
  const [editorOpen, setEditorOpen] = useState(false);
  const [pushingId, setPushingId] = useState<number | null>(null);
  const [pageIds, setPageIds] = useState<Record<number, number>>({});
  const [loopTasks, setLoopTasks] = useState<ImageLoopTask[]>(imageLoopTasks);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingTask, setEditingTask] = useState<ImageLoopTask | undefined>();

  useEffect(() => {
    setLoopTasks(imageLoopTasks);
  }, [imageLoopTasks]);

  const handleTaskUpdate = useCallback((updated: ImageLoopTask) => {
    setLoopTasks((prev) => prev.map((t) => (t.id === updated.id ? updated : t)));
  }, []);

  // Refresh task list from backend
  const refreshTasks = useCallback(async () => {
    await onRefreshLoopTasks();
  }, [onRefreshLoopTasks]);

  useImageLoopRunner(loopTasks, refreshTasks);

  const handleCreateTask = async (input: ImageLoopTaskInput) => {
    await onCreateLoopTask(input);
    await onRefreshLoopTasks();
  };

  const handleUpdateTask = async (input: ImageLoopTaskInput) => {
    if (editingTask) {
      await onUpdateLoopTask(editingTask.id, input);
      await onRefreshLoopTasks();
      setEditingTask(undefined);
    }
  };

  const handleDeleteLoopTask = async (taskId: number) => {
    await onDeleteLoopTask(taskId);
    setLoopTasks((prev) => prev.filter((t) => t.id !== taskId));
  };

  const handleStartTask = async (taskId: number) => {
    const started = await onStartLoopTask(taskId);
    handleTaskUpdate(started);
  };

  const handleStopTask = async (taskId: number) => {
    const stopped = await onStopLoopTask(taskId);
    handleTaskUpdate(stopped);
  };

  const handleEditTask = (task: ImageLoopTask) => {
    setEditingTask(task);
    setDialogOpen(true);
  };

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
      toast.error("没有可用设备");
      return;
    }

    const pageId = pageIds[templateId] ?? 1;
    setPushingId(templateId);
    try {
      await onPushTemplate(templateId, deviceId, pageId);
      toast.success(`推送成功，已发送到第 ${pageId} 页`);
    } catch (e) {
      toast.error(`推送失败: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setPushingId(null);
    }
  }

  async function handleDelete(templateId: number) {
    const confirmed = await confirm("确定要删除这张图片吗？", {
      title: "删除图片",
      kind: "warning",
    });
    if (!confirmed) return;
    try {
      await onDeleteTemplate(templateId);
      setTemplates((prev) => prev.filter((t) => t.id !== templateId));
    } catch (e) {
      toast.error(`删除失败: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  return (
    <section className="space-y-6">
      <header className="rounded-lg bg-gradient-to-r from-violet-50 to-purple-50 px-4 py-3 border border-violet-100">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">本地图库</h2>
            <p className="text-sm text-gray-500">导入后的图片会先保存在应用本地，选中后再单独推送到设备。</p>
          </div>
          <button
            type="button"
            onClick={() => setEditorOpen(true)}
            className="px-4 py-2 bg-violet-500 text-white rounded-md hover:bg-violet-600 focus:outline-none focus:ring-2 focus:ring-violet-500 shadow-sm transition"
          >
            导入图片
          </button>
        </div>
      </header>

      {editorOpen && (
        <ImageEditorDialog onSave={handleSave} onClose={() => setEditorOpen(false)} />
      )}

      <ul className="grid gap-3 grid-cols-5">
        {templates.map((t) => (
          <li key={t.id} className="rounded-xl border border-violet-200 bg-gradient-to-br from-violet-50/30 to-white shadow-sm overflow-hidden hover:shadow-md transition">
            <div className="aspect-[4/3] bg-gray-100 flex items-center justify-center relative">
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
              <button
                type="button"
                onClick={() => handleDelete(t.id)}
                className="absolute top-2 right-2 w-6 h-6 flex items-center justify-center bg-white/90 hover:bg-red-100 text-gray-500 hover:text-red-600 rounded-full shadow-sm transition-colors"
                title="删除图片"
              >
                ✕
              </button>
            </div>
            <div className="flex items-center gap-2 p-2">
              <Select
                value={String(pageIds[t.id] ?? 1)}
                onValueChange={(val) => setPageIds((prev) => ({ ...prev, [t.id]: Number(val) }))}
              >
                <SelectTrigger className="h-8 w-20 px-2 text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {PAGE_OPTIONS.map((opt) => (
                    <SelectItem key={opt.value} value={String(opt.value)} className="text-xs">
                      {opt.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <button
                type="button"
                onClick={() => handlePush(t.id)}
                disabled={pushingId === t.id}
                className="px-3 py-1.5 bg-violet-500 text-white text-sm rounded-md hover:bg-violet-600 disabled:cursor-not-allowed disabled:bg-violet-300 whitespace-nowrap shadow-sm transition"
              >
                {pushingId === t.id ? "推送中..." : "推送"}
              </button>
            </div>
          </li>
        ))}
      </ul>

      {/* 文件夹轮播区域 */}
      <section className="mt-8 space-y-4">
        <header className="rounded-lg bg-gradient-to-r from-purple-50 to-fuchsia-50 px-4 py-3 border border-purple-100">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-lg font-semibold text-gray-900">文件夹轮播</h2>
              <p className="text-sm text-gray-500">设置文件夹图片自动轮播推送到设备。</p>
            </div>
            <button
              type="button"
              onClick={() => {
                setEditingTask(undefined);
                setDialogOpen(true);
              }}
              className="px-4 py-2 bg-purple-500 text-white rounded-md hover:bg-purple-600 shadow-sm transition"
            >
              新建任务
            </button>
          </div>
        </header>

        <ImageLoopTaskList
          tasks={loopTasks}
          devices={devices}
          onStart={handleStartTask}
          onStop={handleStopTask}
          onEdit={handleEditTask}
          onDelete={handleDeleteLoopTask}
        />
      </section>

      <ImageLoopTaskDialog
        open={dialogOpen}
        devices={devices}
        editingTask={editingTask}
        onSave={editingTask ? handleUpdateTask : handleCreateTask}
        onClose={() => {
          setDialogOpen(false);
          setEditingTask(undefined);
        }}
      />
    </section>
  );
}
