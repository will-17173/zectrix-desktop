import { useState } from "react";
import { toast } from "../../components/ui/toast";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "../../components/ui/dialog";
import type {
  BuiltinPlugin,
  CustomPluginInput,
  CustomPluginRecord,
  DeviceRecord,
  PluginLoopTask,
  PluginLoopTaskInput,
} from "../../lib/tauri";

const DEFAULT_PLUGIN_CODE = 'return { type: "text", text: "hello" };';

const PAGE_OPTIONS = [
  { value: 1, label: "第 1 页" },
  { value: 2, label: "第 2 页" },
  { value: 3, label: "第 3 页" },
  { value: 4, label: "第 4 页" },
  { value: 5, label: "第 5 页" },
];

const INTERVAL_OPTIONS = [
  { value: 60, label: "1 分钟" },
  { value: 300, label: "5 分钟" },
  { value: 600, label: "10 分钟" },
  { value: 1800, label: "30 分钟" },
  { value: 3600, label: "60 分钟" },
];

type Props = {
  devices: DeviceRecord[];
  builtinPlugins: BuiltinPlugin[];
  customPlugins: CustomPluginRecord[];
  pluginLoopTasks: PluginLoopTask[];
  onSavePlugin: (input: CustomPluginInput) => Promise<CustomPluginRecord>;
  onDeletePlugin: (pluginId: number) => Promise<void>;
  onPushPlugin: (
    pluginKind: "builtin" | "custom",
    pluginId: string,
    deviceId: string,
    pageId: number,
  ) => Promise<void>;
  onCreateLoopTask: (input: PluginLoopTaskInput) => Promise<PluginLoopTask>;
  onDeleteLoopTask: (taskId: number) => Promise<void>;
  onStartLoopTask: (taskId: number) => Promise<PluginLoopTask>;
  onStopLoopTask: (taskId: number) => Promise<PluginLoopTask>;
};

function createEmptyDraft(): CustomPluginInput {
  return {
    name: "",
    description: "",
    code: DEFAULT_PLUGIN_CODE,
  };
}

function getErrorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function PluginMarketPage({
  devices,
  builtinPlugins,
  customPlugins,
  pluginLoopTasks,
  onSavePlugin,
  onDeletePlugin,
  onPushPlugin,
  onCreateLoopTask,
  onDeleteLoopTask,
  onStartLoopTask,
  onStopLoopTask,
}: Props) {
  const [editing, setEditing] = useState<CustomPluginRecord | null>(null);
  const [draft, setDraft] = useState<CustomPluginInput>(() => createEmptyDraft());
  const [dialogOpen, setDialogOpen] = useState(false);
  const firstDevice = devices[0];

  function handleNewPlugin() {
    setEditing(null);
    setDraft(createEmptyDraft());
    setDialogOpen(true);
  }

  function handleEdit(plugin: CustomPluginRecord) {
    setEditing(plugin);
    setDraft({
      id: plugin.id,
      name: plugin.name,
      description: plugin.description,
      code: plugin.code,
    });
    setDialogOpen(true);
  }

  async function handleSave() {
    try {
      const saved = await onSavePlugin(draft);
      toast.success("插件已保存");
      setEditing(saved);
      setDraft({
        id: saved.id,
        name: saved.name,
        description: saved.description,
        code: saved.code,
      });
    } catch (error) {
      toast.error(`保存失败: ${getErrorMessage(error)}`);
    }
  }

  async function handleDelete(plugin: CustomPluginRecord) {
    try {
      await onDeletePlugin(plugin.id);
      if (editing?.id === plugin.id) {
        setEditing(null);
        setDraft(createEmptyDraft());
      }
      toast.success("插件已删除");
    } catch (error) {
      toast.error(`删除失败: ${getErrorMessage(error)}`);
    }
  }

  async function handleBuiltinPush(plugin: BuiltinPlugin, pageId: number) {
    if (!firstDevice) {
      toast.error("请先在设置中添加设备");
      return;
    }

    try {
      await onPushPlugin("builtin", plugin.id, firstDevice.deviceId, pageId);
      toast.success(`推送成功，已发送到第 ${pageId} 页`);
    } catch (error) {
      toast.error(`推送失败: ${getErrorMessage(error)}`);
    }
  }

  async function handleBuiltinCreateLoop(plugin: BuiltinPlugin, pageId: number, intervalSeconds: number) {
    if (!firstDevice) {
      toast.error("请先在设置中添加设备");
      return;
    }

    try {
      await onCreateLoopTask({
        pluginKind: "builtin",
        pluginId: plugin.id,
        name: `${plugin.name} 循环`,
        deviceId: firstDevice.deviceId,
        pageId,
        intervalSeconds,
        durationType: "none",
      });
      toast.success("循环任务已创建");
    } catch (error) {
      toast.error(`创建循环任务失败: ${getErrorMessage(error)}`);
    }
  }

  async function handlePush(plugin: CustomPluginRecord, pageId: number) {
    if (!firstDevice) {
      toast.error("请先在设置中添加设备");
      return;
    }

    try {
      await onPushPlugin("custom", String(plugin.id), firstDevice.deviceId, pageId);
      toast.success(`推送成功，已发送到第 ${pageId} 页`);
    } catch (error) {
      toast.error(`推送失败: ${getErrorMessage(error)}`);
    }
  }

  async function handleCreateLoop(plugin: CustomPluginRecord, pageId: number, intervalSeconds: number) {
    if (!firstDevice) {
      toast.error("请先在设置中添加设备");
      return;
    }

    try {
      await onCreateLoopTask({
        pluginKind: "custom",
        pluginId: String(plugin.id),
        name: `${plugin.name} 循环`,
        deviceId: firstDevice.deviceId,
        pageId,
        intervalSeconds,
        durationType: "none",
      });
      toast.success("循环任务已创建");
    } catch (error) {
      toast.error(`创建循环任务失败: ${getErrorMessage(error)}`);
    }
  }

  async function handleStartLoopTask(taskId: number) {
    try {
      await onStartLoopTask(taskId);
    } catch (error) {
      toast.error(`启动失败: ${getErrorMessage(error)}`);
    }
  }

  async function handleStopLoopTask(taskId: number) {
    try {
      await onStopLoopTask(taskId);
    } catch (error) {
      toast.error(`停止失败: ${getErrorMessage(error)}`);
    }
  }

  async function handleDeleteLoopTask(taskId: number) {
    try {
      await onDeleteLoopTask(taskId);
    } catch (error) {
      toast.error(`删除失败: ${getErrorMessage(error)}`);
    }
  }

  return (
    <section className="space-y-6">
      <header>
        <h2 className="text-lg font-semibold">插件市场</h2>
        <p className="text-sm text-gray-500">
          插件支持单次推送和创建循环任务，推送到设备指定页面。
        </p>
      </header>

      <section className="space-y-4 rounded-xl border border-gray-200 bg-white/85 p-4 shadow-sm">
        <div>
          <h3 className="text-base font-medium">内置插件</h3>
          <p className="mt-1 text-sm text-gray-500">系统内置插件，支持单次推送和创建循环任务。</p>
        </div>

        {builtinPlugins.length === 0 ? (
          <p className="text-sm text-gray-500">暂无内置插件</p>
        ) : (
          <div className="grid gap-3 lg:grid-cols-2">
            {builtinPlugins.map((plugin) => (
              <PluginCard
                key={plugin.id}
                name={plugin.name}
                description={plugin.description}
                onPush={(pageId) => handleBuiltinPush(plugin, pageId)}
                onCreateLoop={(pageId, intervalSeconds) => handleBuiltinCreateLoop(plugin, pageId, intervalSeconds)}
              />
            ))}
          </div>
        )}
      </section>

      <section className="space-y-4 rounded-xl border border-gray-200 bg-white/85 p-4 shadow-sm">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h3 className="text-base font-medium">自定义插件</h3>
            <p className="mt-1 text-sm text-gray-500">插件返回文本或图片结果，支持单次推送和创建循环任务。</p>
          </div>
          <button
            type="button"
            onClick={handleNewPlugin}
            className="rounded-md border border-gray-300 px-3 py-2 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600"
          >
            新增插件
          </button>
        </div>

        {customPlugins.length === 0 ? (
          <div className="rounded-lg border border-dashed border-gray-300 px-4 py-6 text-sm text-gray-500">
            暂无自定义插件
          </div>
        ) : (
          <div className="grid gap-3 lg:grid-cols-2">
            {customPlugins.map((plugin) => (
              <CustomPluginCard
                key={plugin.id}
                plugin={plugin}
                isEditing={editing?.id === plugin.id}
                onEdit={() => handleEdit(plugin)}
                onPush={(pageId) => handlePush(plugin, pageId)}
                onCreateLoop={(pageId, intervalSeconds) => handleCreateLoop(plugin, pageId, intervalSeconds)}
                onDelete={() => void handleDelete(plugin)}
              />
            ))}
          </div>
        )}
      </section>

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>{editing ? `编辑插件 #${editing.id}` : "新增插件"}</DialogTitle>
          </DialogHeader>

          <div className="space-y-4">
            <div className="space-y-2">
              <label htmlFor="plugin-name" className="block text-sm font-medium text-gray-700">
                插件名称
              </label>
              <input
                id="plugin-name"
                aria-label="插件名称"
                value={draft.name}
                onChange={(event) => setDraft((current) => ({ ...current, name: event.target.value }))}
                className="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="例如：天气播报"
              />
            </div>

            <div className="space-y-2">
              <label htmlFor="plugin-description" className="block text-sm font-medium text-gray-700">
                插件描述
              </label>
              <input
                id="plugin-description"
                aria-label="插件描述"
                value={draft.description}
                onChange={(event) => setDraft((current) => ({ ...current, description: event.target.value }))}
                className="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="说明这个插件会做什么"
              />
            </div>

            <div className="space-y-2">
              <label htmlFor="plugin-code" className="block text-sm font-medium text-gray-700">
                插件代码
              </label>
              <textarea
                id="plugin-code"
                aria-label="插件代码"
                value={draft.code}
                onChange={(event) => setDraft((current) => ({ ...current, code: event.target.value }))}
                className="min-h-56 w-full rounded-md border border-gray-300 px-3 py-2 font-mono text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                spellCheck={false}
              />
            </div>

            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                onClick={() => void handleSave()}
                className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-blue-700"
              >
                保存插件
              </button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      <section className="space-y-3 rounded-xl border border-gray-200 bg-white/85 p-4 shadow-sm">
        <div>
          <h3 className="text-base font-medium">插件循环任务</h3>
          <p className="mt-1 text-sm text-gray-500">已创建的任务会按固定间隔将插件结果推送到设备页面。</p>
        </div>
        {pluginLoopTasks.length === 0 ? (
          <p className="text-sm text-gray-500">暂无循环任务</p>
        ) : (
          <div className="grid gap-3 lg:grid-cols-2">
            {pluginLoopTasks.map((task) => (
              <article key={task.id} className="rounded-lg border border-gray-200 bg-white px-4 py-4 shadow-sm">
                <div className="text-sm font-semibold text-gray-900">{task.name}</div>
                <p className="mt-1 text-sm text-gray-500">
                  {`第 ${task.pageId} 页 · 每 ${task.intervalSeconds} 秒 · ${task.status}`}
                </p>
                {task.errorMessage ? (
                  <p className="mt-2 text-sm text-red-600">{task.errorMessage}</p>
                ) : null}
                <div className="mt-3 flex flex-wrap gap-2">
                  <button
                    type="button"
                    onClick={() => void handleStartLoopTask(task.id)}
                    className="rounded-md border border-gray-300 px-3 py-2 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600"
                  >
                    启动
                  </button>
                  <button
                    type="button"
                    onClick={() => void handleStopLoopTask(task.id)}
                    className="rounded-md border border-gray-300 px-3 py-2 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600"
                  >
                    停止
                  </button>
                  <button
                    type="button"
                    onClick={() => void handleDeleteLoopTask(task.id)}
                    className="rounded-md border border-gray-300 px-3 py-2 text-sm font-medium text-gray-700 transition hover:border-red-400 hover:text-red-600"
                  >
                    删除
                  </button>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>
    </section>
  );
}

type PluginCardProps = {
  name: string;
  description: string;
  onPush: (pageId: number) => Promise<void>;
  onCreateLoop: (pageId: number, intervalSeconds: number) => void;
};

function PluginCard({ name, description, onPush, onCreateLoop }: PluginCardProps) {
  const [pageId, setPageId] = useState(1);
  const [intervalSeconds, setIntervalSeconds] = useState(60);
  const [pushing, setPushing] = useState(false);

  async function handlePush() {
    setPushing(true);
    try {
      await onPush(pageId);
    } finally {
      setPushing(false);
    }
  }

  return (
    <article className="rounded-lg border border-gray-200 bg-white px-4 py-4 shadow-sm">
      <div className="text-sm font-semibold text-gray-900">{name}</div>
      <p className="mt-1 text-sm text-gray-500">{description || "暂无描述"}</p>
      <div className="mt-3 flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-2">
          <Select value={String(pageId)} onValueChange={(v) => setPageId(Number(v))}>
            <SelectTrigger className="w-[106px] h-9">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {PAGE_OPTIONS.map((opt) => (
                <SelectItem key={opt.value} value={String(opt.value)}>
                  {opt.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={String(intervalSeconds)} onValueChange={(v) => setIntervalSeconds(Number(v))}>
            <SelectTrigger className="w-28 h-9">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {INTERVAL_OPTIONS.map((opt) => (
                <SelectItem key={opt.value} value={String(opt.value)}>
                  {opt.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <button
            type="button"
            onClick={() => onCreateLoop(pageId, intervalSeconds)}
            className="rounded-md border border-gray-300 px-3 py-1.5 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600"
          >
            循环
          </button>
        </div>
        <button
          type="button"
          onClick={handlePush}
          disabled={pushing}
          className="rounded-md border border-gray-300 px-3 py-1.5 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {pushing ? "推送中..." : "推送一次"}
        </button>
      </div>
    </article>
  );
}

type CustomPluginCardProps = {
  plugin: CustomPluginRecord;
  isEditing: boolean;
  onEdit: () => void;
  onPush: (pageId: number) => Promise<void>;
  onCreateLoop: (pageId: number, intervalSeconds: number) => void;
  onDelete: () => void;
};

function CustomPluginCard({
  plugin,
  isEditing,
  onEdit,
  onPush,
  onCreateLoop,
  onDelete,
}: CustomPluginCardProps) {
  const [pageId, setPageId] = useState(1);
  const [intervalSeconds, setIntervalSeconds] = useState(60);
  const [pushing, setPushing] = useState(false);

  async function handlePush() {
    setPushing(true);
    try {
      await onPush(pageId);
    } finally {
      setPushing(false);
    }
  }

  return (
    <article
      className={`rounded-lg border px-4 py-4 transition ${
        isEditing
          ? "border-blue-400 bg-blue-50/50 shadow-sm"
          : "border-gray-200 bg-white shadow-sm"
      }`}
    >
      <button
        type="button"
        onClick={onEdit}
        className="w-full text-left"
      >
        <div className="flex items-start justify-between gap-3">
          <div>
            <div className="text-sm font-semibold text-gray-900">{plugin.name}</div>
            <p className="mt-1 text-sm text-gray-500">{plugin.description || "暂无描述"}</p>
          </div>
          {isEditing ? (
            <span className="rounded-full bg-blue-100 px-2 py-1 text-xs font-medium text-blue-700">
              编辑中
            </span>
          ) : null}
        </div>
      </button>
      <div className="mt-3 flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-2">
          <Select value={String(pageId)} onValueChange={(v) => setPageId(Number(v))}>
            <SelectTrigger className="w-[106px] h-9">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {PAGE_OPTIONS.map((opt) => (
                <SelectItem key={opt.value} value={String(opt.value)}>
                  {opt.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={String(intervalSeconds)} onValueChange={(v) => setIntervalSeconds(Number(v))}>
            <SelectTrigger className="w-28 h-9">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {INTERVAL_OPTIONS.map((opt) => (
                <SelectItem key={opt.value} value={String(opt.value)}>
                  {opt.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <button
            type="button"
            onClick={() => onCreateLoop(pageId, intervalSeconds)}
            className="rounded-md border border-gray-300 px-3 py-1.5 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600"
          >
            循环
          </button>
          <button
            type="button"
            onClick={onDelete}
            className="rounded-md border border-gray-300 px-3 py-1.5 text-sm font-medium text-gray-700 transition hover:border-red-400 hover:text-red-600"
          >
            删除
          </button>
        </div>
        <button
          type="button"
          onClick={handlePush}
          disabled={pushing}
          className="rounded-md border border-gray-300 px-3 py-1.5 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {pushing ? "推送中..." : "推送一次"}
        </button>
      </div>
    </article>
  );
}