import { useState } from "react";
import { toast } from "../../components/ui/toast";
import type {
  CustomPluginInput,
  CustomPluginRecord,
  DeviceRecord,
  PluginLoopTask,
  PluginLoopTaskInput,
  PluginRunResult,
} from "../../lib/tauri";

const DEFAULT_PLUGIN_CODE = 'return { type: "text", text: "hello" };';

type Props = {
  devices: DeviceRecord[];
  customPlugins: CustomPluginRecord[];
  pluginLoopTasks: PluginLoopTask[];
  onSavePlugin: (input: CustomPluginInput) => Promise<CustomPluginRecord>;
  onDeletePlugin: (pluginId: number) => Promise<void>;
  onRunPlugin: (pluginKind: "builtin" | "custom", pluginId: string) => Promise<PluginRunResult>;
  onPushPlugin: (
    pluginKind: "builtin" | "custom",
    pluginId: string,
    deviceId: string,
    pageId: number,
  ) => Promise<void>;
  onCreateLoopTask: (input: PluginLoopTaskInput) => Promise<PluginLoopTask>;
  onUpdateLoopTask: (taskId: number, input: PluginLoopTaskInput) => Promise<PluginLoopTask>;
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
  customPlugins,
  pluginLoopTasks,
  onSavePlugin,
  onDeletePlugin,
  onRunPlugin,
  onPushPlugin,
  onCreateLoopTask,
  onDeleteLoopTask,
  onStartLoopTask,
  onStopLoopTask,
}: Props) {
  const [editing, setEditing] = useState<CustomPluginRecord | null>(null);
  const [draft, setDraft] = useState<CustomPluginInput>(() => createEmptyDraft());
  const [runResult, setRunResult] = useState<PluginRunResult | null>(null);
  const firstDevice = devices[0];

  function handleNewPlugin() {
    setEditing(null);
    setDraft(createEmptyDraft());
    setRunResult(null);
  }

  function handleEdit(plugin: CustomPluginRecord) {
    setEditing(plugin);
    setDraft({
      id: plugin.id,
      name: plugin.name,
      description: plugin.description,
      code: plugin.code,
    });
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
        handleNewPlugin();
      }
      toast.success("插件已删除");
    } catch (error) {
      toast.error(`删除失败: ${getErrorMessage(error)}`);
    }
  }

  async function handleRun(plugin: CustomPluginRecord) {
    try {
      const result = await onRunPlugin("custom", String(plugin.id));
      setRunResult(result);
      handleEdit(plugin);
    } catch (error) {
      toast.error(`运行失败: ${getErrorMessage(error)}`);
    }
  }

  async function handlePush(plugin: CustomPluginRecord) {
    if (!firstDevice) {
      toast.error("请先在设置中添加设备");
      return;
    }

    try {
      await onPushPlugin("custom", String(plugin.id), firstDevice.deviceId, 1);
      toast.success("插件已推送");
    } catch (error) {
      toast.error(`推送失败: ${getErrorMessage(error)}`);
    }
  }

  async function handleCreateLoop(plugin: CustomPluginRecord) {
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
        pageId: 1,
        intervalSeconds: 60,
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
      <header className="flex items-start justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold">插件市场</h2>
          <p className="text-sm text-gray-500">
            维护自定义 JS 插件，先测试运行结果，再推送到设备页面。
          </p>
        </div>
      </header>

      <section className="space-y-3 rounded-xl border border-gray-200 bg-white/85 p-4 shadow-sm">
        <div>
          <h3 className="text-base font-medium">内置插件</h3>
          <p className="mt-1 text-sm text-gray-500">暂无内置插件</p>
        </div>
      </section>

      <section className="space-y-4 rounded-xl border border-gray-200 bg-white/85 p-4 shadow-sm">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h3 className="text-base font-medium">自定义插件</h3>
            <p className="mt-1 text-sm text-gray-500">插件返回文本或图片结果，支持测试运行、单次推送和创建循环任务。</p>
          </div>
          <button
            type="button"
            onClick={handleNewPlugin}
            className="rounded-md border border-gray-300 px-3 py-2 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600"
          >
            新建插件
          </button>
        </div>

        {customPlugins.length === 0 ? (
          <div className="rounded-lg border border-dashed border-gray-300 px-4 py-6 text-sm text-gray-500">
            暂无自定义插件
          </div>
        ) : (
          <div className="grid gap-3 lg:grid-cols-2">
            {customPlugins.map((plugin) => {
              const isEditing = editing?.id === plugin.id;

              return (
                <article
                  key={plugin.id}
                  className={`rounded-lg border px-4 py-4 transition ${
                    isEditing
                      ? "border-blue-400 bg-blue-50/50 shadow-sm"
                      : "border-gray-200 bg-white shadow-sm"
                  }`}
                >
                  <button
                    type="button"
                    onClick={() => handleEdit(plugin)}
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
                  <div className="mt-3 flex flex-wrap gap-2">
                    <button
                      type="button"
                      onClick={() => void handleRun(plugin)}
                      className="rounded-md border border-gray-300 px-3 py-2 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600"
                    >
                      测试运行
                    </button>
                    <button
                      type="button"
                      onClick={() => void handlePush(plugin)}
                      className="rounded-md border border-gray-300 px-3 py-2 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600"
                    >
                      推送一次
                    </button>
                    <button
                      type="button"
                      onClick={() => handleCreateLoop(plugin)}
                      className="rounded-md border border-gray-300 px-3 py-2 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600"
                    >
                      创建循环任务
                    </button>
                    <button
                      type="button"
                      onClick={() => void handleDelete(plugin)}
                      className="rounded-md border border-gray-300 px-3 py-2 text-sm font-medium text-gray-700 transition hover:border-red-400 hover:text-red-600"
                    >
                      删除
                    </button>
                  </div>
                </article>
              );
            })}
          </div>
        )}
      </section>

      <section className="space-y-4 rounded-xl border border-gray-200 bg-white/85 p-4 shadow-sm">
        <div>
          <h3 className="text-base font-medium">{editing ? `编辑插件 #${editing.id}` : "插件编辑器"}</h3>
          <p className="mt-1 text-sm text-gray-500">保存后可直接测试运行，推送一次默认使用第一台设备的第 1 页。</p>
        </div>

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
          {editing ? (
            <button
              type="button"
              onClick={handleNewPlugin}
              className="rounded-md border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 transition hover:border-blue-500 hover:text-blue-600"
            >
              取消编辑
            </button>
          ) : null}
        </div>
      </section>

      {runResult ? (
        <section className="space-y-3 rounded-xl border border-gray-200 bg-white/85 p-4 shadow-sm">
          <div>
            <h3 className="text-base font-medium">运行结果</h3>
            <p className="mt-1 text-sm text-gray-500">展示最近一次测试运行的规范化输出。</p>
          </div>

          {runResult.text ? (
            <pre className="overflow-x-auto rounded-lg bg-gray-950 px-4 py-3 text-sm text-gray-100">
              {runResult.text}
            </pre>
          ) : null}

          {runResult.previewPngBase64 ? (
            <img
              src={`data:image/png;base64,${runResult.previewPngBase64}`}
              alt="文本图片预览"
              className="max-h-64 rounded-lg border border-gray-200 bg-white object-contain"
            />
          ) : null}

          {runResult.imageDataUrl ? (
            <img
              src={runResult.imageDataUrl}
              alt="图片预览"
              className="max-h-64 rounded-lg border border-gray-200 bg-white object-contain"
            />
          ) : null}
        </section>
      ) : null}

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