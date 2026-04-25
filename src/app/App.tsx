import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import { AppSidebar } from "../components/layout/app-sidebar";
import { AppToolbar } from "../components/layout/app-toolbar";
import { Toaster, toast } from "../components/ui/toast";
import { FreeLayoutPage } from "../features/free-layout/free-layout-page";
import { ImageTemplatesPage } from "../features/images/image-templates-page";
import { SettingsPage } from "../features/settings/settings-page";
import { SketchPage } from "../features/sketch/sketch-page";
import { StockPushPage } from "../features/stocks/stock-push-page";
import { TextTemplatesPage } from "../features/templates/text-templates-page";
import { PageManagerPage } from "../features/page-manager/page-manager-page";
import { PluginMarketPage } from "../features/plugins/plugin-market-page";
import { TodoListPage } from "../features/todos/todo-list-page";
import {
  addStockWatch,
  addDeviceCache,
  addApiKey,
  createPluginLoopTask,
  removeApiKey,
  createLocalTodo,
  deleteLocalTodo,
  deleteCustomPlugin,
  deleteImageTemplate,
  deletePluginLoopTask,
  getImageThumbnail,
  runPluginOnce,
  loadBootstrapState,
  pushPluginOnce,
  pushFreeLayoutText,
  pushImageTemplate,
  pushSketch,
  saveCustomPlugin,
  pushStockQuotes,
  pushText,
  pushTodoToDevice,
  removeStockWatch,
  removeDeviceCache,
  saveImageTemplate,
  syncAll,
  startPluginLoopTask,
  stopPluginLoopTask,
  toggleTodoStatus,
  updatePluginLoopTask,
  updateLocalTodo,
  createImageLoopTask,
  updateImageLoopTask,
  deleteImageLoopTask,
  startImageLoopTask,
  stopImageLoopTask,
  listImageLoopTasks,
  getStockPushTask,
  createStockPushTask,
  startStockPushTask,
  stopStockPushTask,
  fetchStockQuotes,
  type BootstrapState,
  type ImageLoopTaskInput,
  type StockQuote,
} from "../lib/tauri";
import type { SyncState } from "../features/sync/sync-status";

const emptyState: BootstrapState = {
  apiKeys: [],
  devices: [],
  todos: [],
  textTemplates: [],
  imageTemplates: [],
  imageLoopTasks: [],
  customPlugins: [],
  pluginLoopTasks: [],
  stockWatchlist: [],
  stockPushTask: null,
  lastSyncTime: null,
  pageCache: [],
};

const sectionTitles: Record<string, string> = {
  "/": "待办事项",
  "/sketch-push": "涂鸦推送",
  "/image-push": "图片推送",
  "/free-layout": "自由排版",
  "/stock-push": "股票推送",
  "/text-push": "文本推送",
  "/page-manager": "页面管理",
  "/plugins": "插件市场",
  "/settings": "设置",
};

export default function App() {
  const [state, setState] = useState<BootstrapState>(emptyState);
  const [syncState, setSyncState] = useState<SyncState>("idle");
  const [stockQuotes, setStockQuotes] = useState<StockQuote[]>([]);
  const location = useLocation();

  useEffect(() => {
    void loadBootstrapState().then(setState);
  }, []);

  useEffect(() => {
    document.documentElement.classList.add("theme-shell");
    return () => {
      document.documentElement.classList.remove("theme-shell");
    };
  }, []);

  function reload() {
    void loadBootstrapState().then(setState);
  }

  async function refreshLoopTasks() {
    const tasks = await listImageLoopTasks();
    setState((prev) => ({ ...prev, imageLoopTasks: tasks }));
  }

  async function refreshStockPushTask() {
    const task = await getStockPushTask();
    setState((prev) => ({ ...prev, stockPushTask: task }));
  }

  // Poll stock push task status when running
  useEffect(() => {
    if (state.stockPushTask?.status !== "running" || location.pathname !== "/stock-push") {
      return;
    }

    const interval = setInterval(refreshStockPushTask, 5000);
    return () => clearInterval(interval);
  }, [state.stockPushTask?.status, location.pathname]);

  async function handleSync() {
    setSyncState("syncing");
    toast.loading("正在同步数据…", { id: "sync" });
    try {
      const next = await syncAll();
      setState(next);
      setSyncState("success");
      toast.success("同步成功", { id: "sync" });
    } catch (e) {
      console.error("[sync] 同步失败:", e);
      const errorMsg = e instanceof Error ? e.message : String(e);
      setSyncState(navigator.onLine ? "error" : "offline");
      toast.error(navigator.onLine ? `同步失败: ${errorMsg}` : "当前离线，无法同步", { id: "sync" });
    }
  }

  const path = location.pathname;
  const title = sectionTitles[path] ?? "待办事项";
  const hasApiKey = state.apiKeys.length > 0;

  function renderContent() {
    if (path === "/plugins") {
      return (
        <PluginMarketPage
          devices={state.devices}
          customPlugins={state.customPlugins}
          pluginLoopTasks={state.pluginLoopTasks}
          onSavePlugin={async (input) => {
            const saved = await saveCustomPlugin(input);
            reload();
            return saved;
          }}
          onDeletePlugin={async (pluginId) => {
            await deleteCustomPlugin(pluginId);
            reload();
          }}
          onRunPlugin={(pluginKind, pluginId) =>
            runPluginOnce(pluginKind, pluginId)
          }
          onPushPlugin={(pluginKind, pluginId, deviceId, pageId) =>
            pushPluginOnce(pluginKind, pluginId, deviceId, pageId)
          }
          onCreateLoopTask={async (input) => {
            const task = await createPluginLoopTask(input);
            reload();
            return task;
          }}
          onUpdateLoopTask={async (taskId, input) => {
            const task = await updatePluginLoopTask(taskId, input);
            reload();
            return task;
          }}
          onDeleteLoopTask={async (taskId) => {
            await deletePluginLoopTask(taskId);
            reload();
          }}
          onStartLoopTask={async (taskId) => {
            const task = await startPluginLoopTask(taskId);
            reload();
            return task;
          }}
          onStopLoopTask={async (taskId) => {
            const task = await stopPluginLoopTask(taskId);
            reload();
            return task;
          }}
        />
      );
    }
    if (path === "/settings") {
      return (
        <SettingsPage
          apiKeys={state.apiKeys}
          devices={state.devices}
          onAddApiKey={async (name, key) => {
            const record = await addApiKey(name, key);
            reload();
            return record;
          }}
          onRemoveApiKey={async (id) => {
            await removeApiKey(id);
            reload();
          }}
          onAddDevice={async (id, apiKeyId) => {
            const device = await addDeviceCache(id, apiKeyId);
            reload();
            return device;
          }}
          onRemoveDevice={async (id) => {
            await removeDeviceCache(id);
            reload();
          }}
        />
      );
    }
    if (path === "/page-manager") {
      return (
        <PageManagerPage
          devices={state.devices}
          onRefreshLoopTasks={refreshLoopTasks}
        />
      );
    }
    if (path === "/text-push") {
      return (
        <TextTemplatesPage
          devices={state.devices}
          onPushText={pushText}
        />
      );
    }
    if (path === "/image-push") {
      return (
        <ImageTemplatesPage
          templates={state.imageTemplates}
          devices={state.devices}
          imageLoopTasks={state.imageLoopTasks}
          onSaveTemplate={async (input) => {
            const t = await saveImageTemplate(input);
            reload();
            return t;
          }}
          onPushTemplate={(templateId, deviceId, pageId) =>
            pushImageTemplate(templateId, deviceId, pageId)
          }
          onDeleteTemplate={async (templateId) => {
            await deleteImageTemplate(templateId);
            reload();
          }}
          onLoadThumbnail={getImageThumbnail}
          onCreateLoopTask={async (input: ImageLoopTaskInput) => {
            await createImageLoopTask(input);
          }}
          onUpdateLoopTask={async (taskId: number, input: ImageLoopTaskInput) => {
            await updateImageLoopTask(taskId, input);
          }}
          onDeleteLoopTask={async (taskId: number) => {
            await deleteImageLoopTask(taskId);
          }}
          onStartLoopTask={async (taskId: number) => {
            return await startImageLoopTask(taskId);
          }}
          onStopLoopTask={async (taskId: number) => {
            return await stopImageLoopTask(taskId);
          }}
          onRefreshLoopTasks={refreshLoopTasks}
        />
      );
    }
    if (path === "/sketch-push") {
      return (
        <SketchPage
          devices={state.devices}
          onPushSketch={(dataUrl, deviceId, pageId) =>
            pushSketch(dataUrl, deviceId, pageId)
          }
        />
      );
    }
    if (path === "/free-layout") {
      return (
        <FreeLayoutPage
          devices={state.devices}
          onPushText={pushFreeLayoutText}
        />
      );
    }
    if (path === "/stock-push") {
      return (
        <StockPushPage
          devices={state.devices}
          watchlist={state.stockWatchlist}
          quotes={stockQuotes}
          pushTask={state.stockPushTask ?? null}
          onAddStock={async (code) => {
            const record = await addStockWatch(code);
            setState((prev) => ({ ...prev, stockWatchlist: [...prev.stockWatchlist, record] }));
            return record;
          }}
          onRemoveStock={async (code) => {
            await removeStockWatch(code);
            setState((prev) => ({
              ...prev,
              stockWatchlist: prev.stockWatchlist.filter((stock) => stock.code !== code),
            }));
          }}
          onPushStocks={pushStockQuotes}
          onFetchQuotes={async () => {
            const quotes = await fetchStockQuotes();
            setStockQuotes(quotes);
            return quotes;
          }}
          onCreateTask={async (deviceId, pageId, intervalSeconds) => {
            const task = await createStockPushTask(deviceId, pageId, intervalSeconds);
            setState((prev) => ({ ...prev, stockPushTask: task }));
            return task;
          }}
          onStartTask={async () => {
            const task = await startStockPushTask();
            setState((prev) => ({ ...prev, stockPushTask: task }));
            return task;
          }}
          onStopTask={async () => {
            const task = await stopStockPushTask();
            setState((prev) => ({ ...prev, stockPushTask: task }));
            return task;
          }}
        />
      );
    }
    return (
      <section aria-label="workspace">
        <TodoListPage
          todos={state.todos}
          devices={state.devices}
          syncState={syncState}
          onSync={hasApiKey ? handleSync : undefined}
          onCreateTodo={createLocalTodo}
          onToggleTodo={toggleTodoStatus}
          onDeleteTodo={deleteLocalTodo}
          onUpdateTodo={updateLocalTodo}
          onPushTodo={pushTodoToDevice}
        />
      </section>
    );
  }

  return (
    <div className="app-shell">
      <AppSidebar />
      <main className="app-main">
        <div className="app-main-frame" aria-label="应用工作区外框">
          <AppToolbar title={title} />
          <Toaster position="top-center" />
          <section className="app-canvas" aria-label="主内容画布">
            {renderContent()}
          </section>
        </div>
      </main>
    </div>
  );
}
