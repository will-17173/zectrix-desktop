import { useState } from "react";
import { toast } from "../../components/ui/toast";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import { Checkbox } from "../../components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "../../components/ui/dialog";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "../../components/ui/tabs";
import type {
  BuiltinPlugin,
  PluginConfigOption,
  CustomPluginInput,
  CustomPluginRecord,
  DeviceRecord,
  PluginLoopTask,
  PluginLoopTaskInput,
} from "../../lib/tauri";

const DEFAULT_PLUGIN_CODE = `(async function() {
  // 用户代码开始

  // 用户代码结束
})()`;

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
    config?: Record<string, string>,
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
  const [usageDialogOpen, setUsageDialogOpen] = useState(false);
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
      await onSavePlugin(draft);
      toast.success("插件已保存");
      setEditing(null);
      setDraft(createEmptyDraft());
      setDialogOpen(false);
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

  async function handleBuiltinPush(plugin: BuiltinPlugin, pageId: number, config?: Record<string, string>) {
    if (!firstDevice) {
      toast.error("请先在设置中添加设备");
      return;
    }

    try {
      await onPushPlugin("builtin", plugin.id, firstDevice.deviceId, pageId, config);
      toast.success(`推送成功，已发送到第 ${pageId} 页`);
    } catch (error) {
      toast.error(`推送失败: ${getErrorMessage(error)}`);
    }
  }

  async function handleBuiltinCreateLoop(plugin: BuiltinPlugin, pageId: number, intervalSeconds: number, config?: Record<string, string>) {
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
        config,
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
    <section className="space-y-4">
      <header className="rounded-lg bg-gradient-to-r from-blue-50 to-indigo-50 px-4 py-3 border border-blue-100">
        <h2 className="text-lg font-semibold text-gray-900">插件市场</h2>
        <p className="text-sm text-gray-500">
          插件支持单次推送和创建循环任务，推送到设备指定页面。
        </p>
      </header>

      <Tabs defaultValue="builtin" className="w-full">
        <TabsList className="bg-gray-100/80">
          <TabsTrigger value="builtin" className="data-[state=active]:!bg-blue-500 data-[state=active]:!text-white data-[state=active]:!shadow-sm">内置插件</TabsTrigger>
          <TabsTrigger value="custom" className="data-[state=active]:!bg-indigo-500 data-[state=active]:!text-white data-[state=active]:!shadow-sm">自定义插件</TabsTrigger>
          <TabsTrigger value="tasks" className="data-[state=active]:!bg-emerald-500 data-[state=active]:!text-white data-[state=active]:!shadow-sm">任务管理</TabsTrigger>
        </TabsList>

        <TabsContent value="builtin" className="rounded-xl border border-blue-200 bg-gradient-to-br from-blue-50/50 to-white p-4 shadow-sm">
          <p className="mb-4 text-sm text-gray-500">
            内置插件数量正在开发中，欢迎给作者 Bilibili up{' '}
            <a
              href="https://space.bilibili.com/328381287"
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-600 hover:underline"
            >
              @Terminator-AI
            </a>{' '}
            私信提出开发需求。
          </p>
          {builtinPlugins.length === 0 ? (
            <p className="text-sm text-gray-500">暂无内置插件</p>
          ) : (
            <div className="grid gap-3 lg:grid-cols-2">
              {builtinPlugins.map((plugin) => (
                <PluginCard
                  key={plugin.id}
                  pluginId={plugin.id}
                  name={plugin.name}
                  description={plugin.description}
                  config={plugin.config}
                  supportsLoop={plugin.supportsLoop}
                  onPush={(pageId, config) => handleBuiltinPush(plugin, pageId, config)}
                  onCreateLoop={(pageId, intervalSeconds, config) => handleBuiltinCreateLoop(plugin, pageId, intervalSeconds, config)}
                />
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="custom" className="rounded-xl border border-indigo-200 bg-gradient-to-br from-indigo-50/50 to-white p-4 shadow-sm">
          <div className="mb-4 flex items-center justify-between gap-3">
            <p className="text-sm text-gray-500">管理你自己编写的设备推送插件。</p>
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() => setUsageDialogOpen(true)}
                className="rounded-md border border-indigo-300 px-3 py-2 text-sm font-medium text-indigo-600 transition hover:bg-indigo-50 hover:border-indigo-400"
              >
                使用方法
              </button>
              <button
                type="button"
                onClick={handleNewPlugin}
                className="rounded-md bg-indigo-500 px-3 py-2 text-sm font-medium text-white transition hover:bg-indigo-600 shadow-sm"
              >
                新增插件
              </button>
            </div>
          </div>

          {customPlugins.length === 0 ? (
            <div className="rounded-lg border border-dashed border-indigo-300 bg-indigo-50/30 px-4 py-6 text-sm text-indigo-500">
              暂无自定义插件，点击"新增插件"开始创建
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
        </TabsContent>

        <TabsContent value="tasks" className="rounded-xl border border-emerald-200 bg-gradient-to-br from-emerald-50/50 to-white p-4 shadow-sm">
          <p className="mb-4 text-sm text-gray-500">已创建的任务会按固定间隔将插件结果推送到设备页面。</p>
          {pluginLoopTasks.length === 0 ? (
            <div className="rounded-lg border border-dashed border-emerald-300 bg-emerald-50/30 px-4 py-6 text-sm text-emerald-500">
              暂无循环任务
            </div>
          ) : (
            <div className="grid gap-3 lg:grid-cols-2">
              {pluginLoopTasks.map((task) => (
                <article key={task.id} className="rounded-lg border border-emerald-200 bg-white px-4 py-4 shadow-sm">
                  <div className="flex items-center gap-2">
                    <div className="text-sm font-semibold text-gray-900">{task.name}</div>
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                      task.status === "running"
                        ? "bg-emerald-100 text-emerald-700"
                        : task.status === "idle"
                        ? "bg-gray-100 text-gray-600"
                        : task.status === "error"
                        ? "bg-red-100 text-red-700"
                        : "bg-blue-100 text-blue-700"
                    }`}>
                      {task.status === "running" ? "运行中" : task.status === "idle" ? "已停止" : task.status === "error" ? "错误" : "已完成"}
                    </span>
                  </div>
                  <p className="mt-1 text-sm text-gray-500">
                    {`第 ${task.pageId} 页 · 每 ${Math.floor(task.intervalSeconds / 60)} 分钟`}
                  </p>
                  {task.errorMessage ? (
                    <p className="mt-2 text-sm text-red-600">{task.errorMessage}</p>
                  ) : null}
                  <div className="mt-3 flex flex-wrap gap-2">
                    <button
                      type="button"
                      onClick={() => void handleStartLoopTask(task.id)}
                      className="rounded-md bg-emerald-500 px-3 py-1.5 text-sm font-medium text-white transition hover:bg-emerald-600 shadow-sm"
                    >
                      启动
                    </button>
                    <button
                      type="button"
                      onClick={() => void handleStopLoopTask(task.id)}
                      className="rounded-md border border-amber-300 px-3 py-1.5 text-sm font-medium text-amber-600 transition hover:bg-amber-50"
                    >
                      停止
                    </button>
                    <button
                      type="button"
                      onClick={() => void handleDeleteLoopTask(task.id)}
                      className="rounded-md border border-red-300 px-3 py-1.5 text-sm font-medium text-red-600 transition hover:bg-red-50"
                    >
                      删除
                    </button>
                  </div>
                </article>
              ))}
            </div>
          )}
        </TabsContent>
      </Tabs>

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

      <Dialog open={usageDialogOpen} onOpenChange={setUsageDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>自定义插件使用方法</DialogTitle>
          </DialogHeader>

          <div className="max-h-[70vh] space-y-4 overflow-y-auto pr-1 text-sm leading-6 text-gray-600">
            <section className="space-y-2">
              <h3 className="text-sm font-semibold text-gray-900">执行方式</h3>
              <p>插件代码会在异步函数中执行，可以返回文本或图片结果。</p>
              <p>你可以在代码里使用标准 JavaScript 语法，适合生成固定文案、拼接时间信息、请求接口后整理成设备可读内容。</p>
            </section>

            <section className="space-y-2">
              <h3 className="text-sm font-semibold text-gray-900">返回格式</h3>
              <p>
                返回文本时使用 <code className="rounded bg-gray-100 px-1 py-0.5 font-mono text-xs">{"{ type: \"text\", text: \"内容\" }"}</code>。
              </p>
              <p>
                返回图片时使用 <code className="rounded bg-gray-100 px-1 py-0.5 font-mono text-xs">{"{ type: \"image\", image: \"data:image/png;base64,...\" }"}</code>。
              </p>
              <p>
                如果图片在网上，也可以返回 <code className="rounded bg-gray-100 px-1 py-0.5 font-mono text-xs">{"{ type: \"image\", imageUrl: \"https://example.com/card.png\" }"}</code>，
                应用会先下载图片，再转换为设备需要的 PNG 推送。
              </p>
              <p>返回值必须是对象；如果接口请求失败，建议返回一段可读的错误文本，方便在设备页面上排查。</p>
            </section>

            <section className="space-y-2">
              <h3 className="text-sm font-semibold text-gray-900">文本示例</h3>
              <pre className="overflow-x-auto rounded-lg bg-gray-950 px-3 py-3 text-xs leading-5 text-gray-100">
                <code>{`const now = new Date().toLocaleString();
return {
  type: "text",
  text: \`当前时间：\${now}\`,
};`}</code>
              </pre>
            </section>

            <section className="space-y-2">
              <h3 className="text-sm font-semibold text-gray-900">接口请求示例</h3>
              <pre className="overflow-x-auto rounded-lg bg-gray-950 px-3 py-3 text-xs leading-5 text-gray-100">
                <code>{`const response = await fetch("https://example.com/api/status");
const data = await response.json();

return {
  type: "text",
  text: \`状态：\${data.message}\`,
};`}</code>
              </pre>
            </section>

            <section className="space-y-2">
              <h3 className="text-sm font-semibold text-gray-900">图片 URL 示例</h3>
              <pre className="overflow-x-auto rounded-lg bg-gray-950 px-3 py-3 text-xs leading-5 text-gray-100">
                <code>{`return {
  type: "image",
  imageUrl: "https://example.com/card.png",
};`}</code>
              </pre>
            </section>

            <section className="space-y-2">
              <h3 className="text-sm font-semibold text-gray-900">循环任务注意事项</h3>
              <p>保存后可以在自定义插件列表里单次推送，也可以选择页面和间隔创建循环任务。</p>
              <p>循环任务会反复执行同一段插件代码，请避免写入耗时过长、频率过高或依赖不稳定接口的逻辑。</p>
              <p>如果插件需要访问网络，请确认接口返回稳定，并给设备页面准备一个简短、可读的失败提示。</p>
            </section>
          </div>
        </DialogContent>
      </Dialog>
    </section>
  );
}

type PluginCardProps = {
  pluginId: string;
  name: string;
  description: string;
  config?: PluginConfigOption[];
  supportsLoop?: boolean;
  onPush: (pageId: number, config?: Record<string, string>) => Promise<void>;
  onCreateLoop: (pageId: number, intervalSeconds: number, config?: Record<string, string>) => void;
};

// 配置项分组：哪些放在配置对话框里，哪些直接显示
const CONFIG_HIDDEN_KEYS = [
  "comfyuiUrl", "workflow", "promptNodeId", "promptField", "seedNodeId", "seedField", "randomizeSeed",
  // GitHub Actions 插件的配置项放到配置弹窗里
  "token", "repo",
];

// 使用插件 ID 作为 localStorage key，更稳定
function getStorageKey(pluginId: string) {
  return `plugin-config-${pluginId}`;
}

function PluginCard({ name, description, config, supportsLoop = true, onPush, onCreateLoop, pluginId }: PluginCardProps & { pluginId: string }) {
  const [pageId, setPageId] = useState(1);
  const [intervalSeconds, setIntervalSeconds] = useState(60);
  const [pushing, setPushing] = useState(false);
  const [configDialogOpen, setConfigDialogOpen] = useState(false);
  // 从 localStorage 加载配置，使用默认值填充
  const [configValues, setConfigValues] = useState<Record<string, string>>(() => {
    if (!config) return {};
    const saved = localStorage.getItem(getStorageKey(pluginId));
    const savedValues = saved ? JSON.parse(saved) : {};
    return config.reduce((acc, opt) => {
      acc[opt.name] = savedValues[opt.name] ?? opt.default;
      return acc;
    }, {} as Record<string, string>);
  });

  // 分离可见配置和隐藏配置
  const visibleConfig = config?.filter(opt => !CONFIG_HIDDEN_KEYS.includes(opt.name)) || [];
  const hiddenConfig = config?.filter(opt => CONFIG_HIDDEN_KEYS.includes(opt.name)) || [];
  const hasHiddenConfig = hiddenConfig.length > 0;

  // 保存配置到 localStorage
  function updateConfigValue(key: string, value: string) {
    setConfigValues((prev) => {
      const next = { ...prev, [key]: value };
      localStorage.setItem(getStorageKey(pluginId), JSON.stringify(next));
      return next;
    });
  }

  async function handlePush() {
    setPushing(true);
    try {
      await onPush(pageId, configValues);
    } finally {
      setPushing(false);
    }
  }

  function handleCreateLoop() {
    onCreateLoop(pageId, intervalSeconds, configValues);
  }

  return (
    <article className="flex flex-col rounded-lg border border-blue-200 bg-white px-4 py-4 shadow-sm hover:shadow-md transition">
      <div className="flex-1">
        <div className="flex items-center gap-2">
          <div className="w-1.5 h-1.5 rounded-full bg-blue-500"></div>
          <div className="text-sm font-semibold text-gray-900">{name}</div>
          {hasHiddenConfig && (
            <button
              type="button"
              onClick={() => setConfigDialogOpen(true)}
              className="rounded-md border border-gray-300 px-2 py-0.5 text-xs font-medium text-gray-600 transition hover:bg-gray-50"
            >
              配置
            </button>
          )}
        </div>
        <p className="mt-1 text-sm text-gray-500 pl-3.5">{description || "暂无描述"}</p>
        {/* 只显示可见的配置选项 */}
        {visibleConfig.length > 0 && (
          <div className="mt-2 space-y-2">
          {visibleConfig.map((opt) => {
            const isInput = opt.inputType && opt.inputType !== "";
            const inputType = opt.inputType === "password" ? "password" : "text";
            if (isInput) {
              return (
                <div key={opt.name} className="flex items-center gap-2">
                  <span className="text-xs text-gray-500 shrink-0">{opt.label}</span>
                  <input
                    type={inputType}
                    value={configValues[opt.name] || opt.default}
                    onChange={(e) => updateConfigValue(opt.name, e.target.value)}
                    className="flex-1 h-9 rounded-md border border-gray-300 px-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              );
            }
            return (
              <div key={opt.name} className="flex items-center gap-1.5">
                <span className="text-xs text-gray-500">{opt.label}</span>
                <Select
                  value={configValues[opt.name] || opt.default}
                  onValueChange={(v) => updateConfigValue(opt.name, v)}
                >
                  <SelectTrigger className="w-[100px] h-9">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {(opt.options || []).map((item) => (
                      <SelectItem key={item.value} value={item.value}>
                        {item.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            );
          })}
        </div>
      )}
      </div>
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
          {supportsLoop && (
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
          )}
          {supportsLoop && (
            <button
              type="button"
              onClick={handleCreateLoop}
              className="rounded-md border border-blue-300 px-3 py-1.5 text-sm font-medium text-blue-600 transition hover:bg-blue-50"
            >
              循环
            </button>
          )}
        </div>
        <button
          type="button"
          onClick={handlePush}
          disabled={pushing}
          className="rounded-md bg-blue-500 px-3 py-1.5 text-sm font-medium text-white transition hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm"
        >
          {pushing ? "推送中..." : "推送一次"}
        </button>
      </div>

      {/* 配置对话框 */}
      <Dialog open={configDialogOpen} onOpenChange={setConfigDialogOpen}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>{name} - 配置</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            {hiddenConfig.map((opt) => {
              const isTextarea = opt.inputType === "textarea";
              const isCheckbox = opt.inputType === "checkbox";
              const isSelect = !opt.inputType && opt.options && opt.options.length > 0;
              const inputType = opt.inputType === "password" ? "password" : "text";
              if (isTextarea) {
                return (
                  <div key={opt.name} className="space-y-2">
                    <label className="text-sm font-medium text-gray-700">{opt.label}</label>
                    <textarea
                      value={configValues[opt.name] || opt.default}
                      onChange={(e) => updateConfigValue(opt.name, e.target.value)}
                      className="w-full min-h-32 rounded-md border border-gray-300 px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 resize-y"
                      spellCheck={false}
                      placeholder="粘贴从 ComfyUI 导出的工作流 JSON..."
                    />
                  </div>
                );
              }
              if (isCheckbox) {
                const checked = configValues[opt.name] === "true";
                return (
                  <div key={opt.name} className="flex items-center gap-2">
                    <label className="text-sm font-medium text-gray-700 shrink-0 w-28">{opt.label}</label>
                    <Checkbox
                      checked={checked}
                      onCheckedChange={(v) => updateConfigValue(opt.name, v ? "true" : "false")}
                    />
                  </div>
                );
              }
              if (isSelect) {
                return (
                  <div key={opt.name} className="flex items-center gap-2">
                    <label className="text-sm font-medium text-gray-700 shrink-0 w-28">{opt.label}</label>
                    <Select
                      value={configValues[opt.name] || opt.default}
                      onValueChange={(v) => updateConfigValue(opt.name, v)}
                    >
                      <SelectTrigger className="w-[120px] h-9">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {(opt.options || []).map((item) => (
                          <SelectItem key={item.value} value={item.value}>
                            {item.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                );
              }
              return (
                <div key={opt.name} className="flex items-center gap-2">
                  <label className="text-sm font-medium text-gray-700 shrink-0 w-28">{opt.label}</label>
                  <input
                    type={inputType}
                    value={configValues[opt.name] || opt.default}
                    onChange={(e) => updateConfigValue(opt.name, e.target.value)}
                    className="flex-1 h-9 rounded-md border border-gray-300 px-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              );
            })}
            <div className="flex justify-end">
              <button
                type="button"
                onClick={() => setConfigDialogOpen(false)}
                className="rounded-md bg-blue-500 px-4 py-2 text-sm font-medium text-white transition hover:bg-blue-600"
              >
                完成
              </button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
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
      className={`rounded-lg border px-4 py-4 transition shadow-sm hover:shadow-md ${
        isEditing
          ? "border-indigo-400 bg-indigo-50/80 ring-2 ring-indigo-200"
          : "border-indigo-200 bg-white"
      }`}
    >
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-2">
            <div className={`w-1.5 h-1.5 rounded-full ${isEditing ? "bg-indigo-500" : "bg-indigo-400"}`}></div>
            <div className="text-sm font-semibold text-gray-900">{plugin.name}</div>
          </div>
          <p className="mt-1 text-sm text-gray-500 pl-3.5">{plugin.description || "暂无描述"}</p>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          {isEditing ? (
            <span className="rounded-full bg-indigo-100 px-2 py-1 text-xs font-medium text-indigo-700">
              编辑中
            </span>
          ) : null}
          <button
            type="button"
            onClick={onEdit}
            className="rounded-md border border-indigo-300 px-3 py-1.5 text-sm font-medium text-indigo-600 transition hover:bg-indigo-50"
          >
            编辑
          </button>
        </div>
      </div>
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
            className="rounded-md border border-indigo-300 px-3 py-1.5 text-sm font-medium text-indigo-600 transition hover:bg-indigo-50"
          >
            循环
          </button>
          <button
            type="button"
            onClick={onDelete}
            className="rounded-md border border-red-300 px-3 py-1.5 text-sm font-medium text-red-600 transition hover:bg-red-50"
          >
            删除
          </button>
        </div>
        <button
          type="button"
          onClick={handlePush}
          disabled={pushing}
          className="rounded-md bg-indigo-500 px-3 py-1.5 text-sm font-medium text-white transition hover:bg-indigo-600 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm"
        >
          {pushing ? "推送中..." : "推送一次"}
        </button>
      </div>
    </article>
  );
}
