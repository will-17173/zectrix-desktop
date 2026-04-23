import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import { AppSidebar } from "../components/layout/app-sidebar";
import { AppToolbar } from "../components/layout/app-toolbar";
import { FreeLayoutPage } from "../features/free-layout/free-layout-page";
import { ImageTemplatesPage } from "../features/images/image-templates-page";
import { SettingsPage } from "../features/settings/settings-page";
import { SketchPage } from "../features/sketch/sketch-page";
import { TextTemplatesPage } from "../features/templates/text-templates-page";
import { TodoListPage } from "../features/todos/todo-list-page";
import {
  addDeviceCache,
  addApiKey,
  removeApiKey,
  createLocalTodo,
  createTextTemplate,
  deleteLocalTodo,
  deleteImageTemplate,
  getImageThumbnail,
  loadBootstrapState,
  pushImageTemplate,
  pushSketch,
  pushText,
  pushTextTemplate,
  pushTodoToDevice,
  removeDeviceCache,
  saveImageTemplate,
  syncAll,
  toggleTodoStatus,
  updateLocalTodo,
  type BootstrapState,
} from "../lib/tauri";
import type { SyncState } from "../features/sync/sync-status";

const emptyState: BootstrapState = {
  apiKeys: [],
  devices: [],
  todos: [],
  textTemplates: [],
  imageTemplates: [],
  lastSyncTime: null,
};

const sectionTitles: Record<string, string> = {
  "/": "待办事项",
  "/sketch-push": "涂鸦推送",
  "/image-push": "图片推送",
  "/free-layout": "自由排版",
  "/text-push": "文本推送",
  "/settings": "设置",
};

export default function App() {
  const [state, setState] = useState<BootstrapState>(emptyState);
  const [syncState, setSyncState] = useState<SyncState>("idle");
  const [syncMessage, setSyncMessage] = useState<string | null>(null);
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

  // 同步成功或失败后 3 秒清除状态
  useEffect(() => {
    if (syncState === "success" || syncState === "error" || syncState === "offline") {
      const timer = setTimeout(() => {
        setSyncState("idle");
        setSyncMessage(null);
      }, 3000);
      return () => clearTimeout(timer);
    }
  }, [syncState]);

  function reload() {
    void loadBootstrapState().then(setState);
  }

  async function handleSync() {
    setSyncState("syncing");
    setSyncMessage("正在同步数据…");
    try {
      const next = await syncAll();
      setState(next);
      setSyncState("success");
      setSyncMessage("同步成功");
    } catch (e) {
      console.error("[sync] 同步失败:", e);
      const errorMsg = e instanceof Error ? e.message : String(e);
      setSyncState(navigator.onLine ? "error" : "offline");
      setSyncMessage(navigator.onLine ? `同步失败: ${errorMsg}` : "当前离线，无法同步");
    }
  }

  const path = location.pathname;
  const title = sectionTitles[path] ?? "待办事项";
  const hasApiKey = state.apiKeys.length > 0;

  function renderContent() {
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
    if (path === "/text-push") {
      return (
        <TextTemplatesPage
          templates={state.textTemplates}
          devices={state.devices}
          onCreateTemplate={async (input) => {
            const t = await createTextTemplate(input);
            reload();
            return t;
          }}
          onPushTemplate={pushTextTemplate}
        />
      );
    }
    if (path === "/image-push") {
      return (
        <ImageTemplatesPage
          templates={state.imageTemplates}
          devices={state.devices}
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
          onPushText={pushText}
        />
      );
    }
    return (
      <section aria-label="workspace">
        <TodoListPage
          todos={state.todos}
          devices={state.devices}
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
          <AppToolbar
            title={title}
            syncState={syncState}
            syncMessage={syncMessage}
            onSync={hasApiKey ? handleSync : undefined}
          />
          <section className="app-canvas" aria-label="主内容画布">
            {renderContent()}
          </section>
        </div>
      </main>
    </div>
  );
}
