import { invoke } from "@tauri-apps/api/core";

export type ApiKeyRecord = {
  id: number;
  name: string;
  key: string;
  createdAt: string;
};

export type ImageLoopTask = {
  id: number;
  name: string;
  folderPath: string;
  deviceId: string;
  pageId: number;
  intervalSeconds: number;
  durationType: "none" | "until_time" | "for_duration";
  endTime?: string;
  durationMinutes?: number;
  status: "idle" | "running" | "completed" | "error";
  currentIndex: number;
  totalImages: number;
  startedAt?: string;
  lastPushAt?: string;
  errorMessage?: string;
  createdAt: string;
  updatedAt: string;
};

export type ImageLoopTaskInput = {
  name: string;
  folderPath: string;
  deviceId: string;
  pageId: number;
  intervalSeconds: number;
  durationType: "none" | "until_time" | "for_duration";
  endTime?: string;
  durationMinutes?: number;
};

export type ImageFolderScanResult = {
  totalImages: number;
  imageFiles: string[];
  warning?: string;
};

export type BootstrapState = {
  apiKeys: ApiKeyRecord[];
  devices: DeviceRecord[];
  todos: Array<TodoRecord>;
  textTemplates: Array<TextTemplateRecord>;
  imageTemplates: Array<ImageTemplateRecord>;
  lastSyncTime: string | null;
  pageCache: Array<PageCacheRecord>;
  imageLoopTasks: Array<ImageLoopTask>;
};

export async function loadBootstrapState(): Promise<BootstrapState> {
  return invoke<BootstrapState>("load_bootstrap_state");
}

export async function listApiKeys(): Promise<ApiKeyRecord[]> {
  return invoke<ApiKeyRecord[]>("list_api_keys");
}

export async function addApiKey(name: string, key: string): Promise<ApiKeyRecord> {
  return invoke<ApiKeyRecord>("add_api_key", { name, key });
}

export async function removeApiKey(id: number): Promise<void> {
  return invoke("remove_api_key", { id });
}

export type DeviceRecord = {
  deviceId: string;
  alias: string;
  board: string;
  cachedAt: string;
  apiKeyId: number;
};

export async function addDeviceCache(deviceId: string, apiKeyId: number): Promise<DeviceRecord> {
  return invoke<DeviceRecord>("add_device_cache", { deviceId, apiKeyId });
}

export async function removeDeviceCache(deviceId: string): Promise<void> {
  return invoke("remove_device_cache", { deviceId });
}

export type TodoRecord = {
  localId: string;
  id: number | null;
  title: string;
  description: string;
  dueDate: string | null;
  dueTime: string | null;
  repeatType: string | null;
  repeatWeekday: number | null;
  repeatMonth: number | null;
  repeatDay: number | null;
  status: number;
  priority: number;
  deviceId: string | null;
  dirty: boolean;
  deleted: boolean;
  createdAt: string;
  updatedAt: string;
};

export type TodoUpsertInput = {
  title: string;
  description?: string;
  dueDate?: string;
  dueTime?: string;
  repeatType?: string;
  repeatWeekday?: number;
  repeatMonth?: number;
  repeatDay?: number;
  priority?: number;
  deviceId?: string;
};

export async function createLocalTodo(input: TodoUpsertInput): Promise<TodoRecord> {
  return invoke<TodoRecord>("create_local_todo", { input });
}

export async function toggleTodoStatus(localId: string): Promise<TodoRecord> {
  return invoke<TodoRecord>("toggle_todo_status", { localId });
}

export async function deleteLocalTodo(localId: string): Promise<void> {
  return invoke("delete_local_todo", { localId });
}

export async function updateLocalTodo(
  localId: string,
  input: TodoUpsertInput
): Promise<TodoRecord> {
  return invoke<TodoRecord>("update_local_todo", { localId, input });
}

export async function pushTodoToDevice(localId: string, deviceId: string, pageId?: number): Promise<void> {
  console.log("[tauri] pushTodoToDevice 调用", { localId, deviceId, pageId });
  try {
    const result = await invoke<void>("push_todo_to_device", { localId, deviceId, pageId });
    console.log("[tauri] pushTodoToDevice 返回", result);
    return result;
  } catch (e) {
    console.error("[tauri] pushTodoToDevice 错误", e);
    throw e;
  }
}

export async function syncAll(): Promise<BootstrapState> {
  return invoke<BootstrapState>("sync_all");
}

export type TextTemplateRecord = {
  id: number;
  title: string;
  content: string;
};

export type TextTemplateInput = {
  title: string;
  content: string;
};

export async function createTextTemplate(input: TextTemplateInput): Promise<TextTemplateRecord> {
  return invoke<TextTemplateRecord>("create_text_template", { input });
}

export async function pushTextTemplate(templateId: number, deviceId: string, pageId?: number): Promise<void> {
  return invoke("push_text_template", { templateId, deviceId, pageId });
}

export type ImageTemplateRecord = {
  id: number;
  name: string;
  filePath: string;
};

export type ImageTemplateSaveInput = {
  name: string;
  sourcePath?: string;
  sourceDataUrl?: string;
  crop: { x: number; y: number; width: number; height: number };
  rotation: number;
  flipX: boolean;
  flipY: boolean;
};

export async function saveImageTemplate(input: ImageTemplateSaveInput): Promise<ImageTemplateRecord> {
  return invoke<ImageTemplateRecord>("save_image_template", { input });
}

export async function pushImageTemplate(templateId: number, deviceId: string, pageId: number): Promise<void> {
  return invoke("push_image_template", { templateId, deviceId, pageId });
}

export async function getImageThumbnail(templateId: number): Promise<string> {
  return invoke<string>("get_image_thumbnail", { templateId });
}

export async function deleteImageTemplate(templateId: number): Promise<void> {
  return invoke("delete_image_template", { templateId });
}

export async function renderImagePreview(input: ImageTemplateSaveInput): Promise<string> {
  return invoke<string>("render_image_preview", { input });
}

export async function pushSketch(dataUrl: string, deviceId: string, pageId: number): Promise<void> {
  return invoke("push_sketch", { dataUrl, deviceId, pageId });
}

export async function pushText(
  title: string,
  body: string,
  deviceId: string,
  pageId: number
): Promise<void> {
  return invoke("push_structured_text", { title, body, deviceId, pageId });
}

export async function pushFreeLayoutText(
  text: string,
  fontSize: number,
  deviceId: string,
  pageId: number
): Promise<void> {
  return invoke("push_text", { text, fontSize, deviceId, pageId });
}

export type UpdateInfo = {
  current_version: string;
  latest_version: string;
  has_update: boolean;
  release_url: string;
  release_notes: string | null;
};

export async function checkForUpdate(): Promise<UpdateInfo> {
  return invoke<UpdateInfo>("check_for_update");
}

export async function getCurrentVersion(): Promise<string> {
  return invoke<string>("get_current_version");
}

export type PageCacheRecord = {
  deviceId: string;
  pageId: number;
  contentType: "sketch" | "image" | "text" | "structured_text";
  thumbnail: string | null;
  metadata: Record<string, unknown> | null;
  pushedAt: string;
};

export async function getPageCacheList(deviceId: string): Promise<PageCacheRecord[]> {
  return invoke<PageCacheRecord[]>("get_page_cache_list", { deviceId });
}

export async function deletePageCache(deviceId: string, pageId: number): Promise<void> {
  return invoke("delete_page_cache", { deviceId, pageId });
}

export async function listImageLoopTasks(): Promise<ImageLoopTask[]> {
  return invoke<ImageLoopTask[]>("list_image_loop_tasks");
}

export async function createImageLoopTask(input: ImageLoopTaskInput): Promise<ImageLoopTask> {
  return invoke<ImageLoopTask>("create_image_loop_task", { input });
}

export async function updateImageLoopTask(taskId: number, input: ImageLoopTaskInput): Promise<ImageLoopTask> {
  return invoke<ImageLoopTask>("update_image_loop_task", { taskId, input });
}

export async function deleteImageLoopTask(taskId: number): Promise<void> {
  return invoke("delete_image_loop_task", { taskId });
}

export async function startImageLoopTask(taskId: number): Promise<ImageLoopTask> {
  return invoke<ImageLoopTask>("start_image_loop_task", { taskId });
}

export async function stopImageLoopTask(taskId: number): Promise<ImageLoopTask> {
  return invoke<ImageLoopTask>("stop_image_loop_task", { taskId });
}

export async function pushFolderImage(taskId: number): Promise<ImageLoopTask> {
  return invoke<ImageLoopTask>("push_folder_image", { taskId });
}

export async function scanImageFolder(folderPath: string): Promise<ImageFolderScanResult> {
  return invoke<ImageFolderScanResult>("scan_image_folder", { folderPath });
}

export async function selectFolderDialog(): Promise<string | null> {
  return invoke<string | null>("select_folder_dialog");
}
