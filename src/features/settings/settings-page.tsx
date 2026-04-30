import { FormEvent, useEffect, useState } from "react";
import { type ApiKeyRecord, type DeviceRecord, checkForUpdate, getCurrentVersion, type UpdateInfo } from "../../lib/tauri";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
import { BilibiliIcon, HeartIcon } from "../../components/layout/app-sidebar";
import { openUrl } from "@tauri-apps/plugin-opener";
import { CalendarSyncPanel } from "./calendar-sync-panel";

type SettingsPageProps = {
  apiKeys: ApiKeyRecord[];
  devices: DeviceRecord[];
  onAddApiKey: (name: string, key: string) => Promise<ApiKeyRecord>;
  onRemoveApiKey: (id: number) => Promise<void>;
  onAddDevice: (deviceId: string, apiKeyId: number) => Promise<DeviceRecord>;
  onRemoveDevice: (deviceId: string) => Promise<void>;
};

const macPattern = /^[0-9A-F]{2}(:[0-9A-F]{2}){5}$/;

export function SettingsPage({
  apiKeys: initialApiKeys,
  devices: initialDevices,
  onAddApiKey,
  onRemoveApiKey,
  onAddDevice,
  onRemoveDevice,
}: SettingsPageProps) {
  const [apiKeyName, setApiKeyName] = useState("");
  const [apiKeyValue, setApiKeyValue] = useState("");
  const [apiKeys, setApiKeys] = useState<ApiKeyRecord[]>(initialApiKeys);
  const [deviceId, setDeviceId] = useState("");
  const [selectedApiKey, setSelectedApiKey] = useState("");
  const [devices, setDevices] = useState<DeviceRecord[]>(initialDevices);
  const [apiKeyError, setApiKeyError] = useState<string | null>(null);
  const [deviceError, setDeviceError] = useState<string | null>(null);
  const [showApiKeyForm, setShowApiKeyForm] = useState(initialApiKeys.length === 0);
  const [showDeviceForm, setShowDeviceForm] = useState(initialDevices.length === 0);
  const [currentVersion, setCurrentVersion] = useState<string>("");
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);

  // 加载当前版本
  useEffect(() => {
    getCurrentVersion().then(setCurrentVersion);
  }, []);

  async function handleCheckUpdate() {
    setCheckingUpdate(true);
    setUpdateError(null);
    setUpdateInfo(null);
    try {
      const info = await checkForUpdate();
      setUpdateInfo(info);
    } catch (e) {
      setUpdateError(e instanceof Error ? e.message : String(e));
    } finally {
      setCheckingUpdate(false);
    }
  }

  async function handleOpenRelease() {
    if (updateInfo?.release_url) {
      await openUrl(updateInfo.release_url);
    }
  }

  async function handleAddApiKey(event: FormEvent) {
    event.preventDefault();
    if (!apiKeyName.trim() || !apiKeyValue.trim()) {
      setApiKeyError("请填写名称和 API Key");
      return;
    }
    try {
      const record = await onAddApiKey(apiKeyName.trim(), apiKeyValue.trim());
      setApiKeys((prev) => [...prev, record]);
      setApiKeyName("");
      setApiKeyValue("");
      setShowApiKeyForm(false);
      setApiKeyError(null);
    } catch (e) {
      setApiKeyError(String(e));
    }
  }

  async function handleRemoveApiKey(id: number) {
    try {
      await onRemoveApiKey(id);
      setApiKeys((prev) => prev.filter((k) => k.id !== id));
      setApiKeyError(null);
    } catch (e) {
      setApiKeyError(String(e));
    }
  }

  async function handleAddDevice() {
    if (!selectedApiKey) {
      setDeviceError("请先选择 API Key");
      return;
    }
    const normalized = deviceId.trim().toUpperCase();
    if (!macPattern.test(normalized)) {
      setDeviceError("MAC 地址格式错误");
      return;
    }
    try {
      const device = await onAddDevice(normalized, Number(selectedApiKey));
      setDevices((prev) => [...prev, device]);
      setDeviceId("");
      setSelectedApiKey("");
      setShowDeviceForm(false);
      setDeviceError(null);
    } catch (e) {
      setDeviceError(String(e));
    }
  }

  async function handleRemoveDevice(id: string) {
    await onRemoveDevice(id);
    setDevices((prev) => prev.filter((d) => d.deviceId !== id));
  }

  const maskApiKey = (key: string) => {
    if (key.length <= 10) return key;
    const prefix = key.slice(0, 6);
    const suffix = key.slice(-4);
    return `${prefix}***${suffix}`;
  };

  const getApiKeyName = (apiKeyId: number) => {
    const key = apiKeys.find((k) => k.id === apiKeyId);
    return key?.name ?? "未知";
  };

  return (
    <section className="p-4 flex flex-col min-h-full">
      <header className="rounded-lg bg-gradient-to-r from-gray-50 to-slate-50 px-4 py-3 border border-gray-200 mb-6">
        <h2 className="text-xl font-semibold text-gray-900">设置</h2>
        <p className="text-sm text-gray-500">管理 API Key、设备和应用配置。</p>
      </header>
      <div className="flex-1 space-y-8">
        <div>
        <div className="flex items-center justify-between gap-3 mb-4">
          <div>
            <h3 className="text-lg font-semibold text-gray-900">API Key 管理</h3>
            <p className="text-sm text-gray-500">用于与云端服务通信的认证密钥。</p>
          </div>
          {!showApiKeyForm && (
            <button
              type="button"
              onClick={() => setShowApiKeyForm(true)}
              className="px-4 py-2 bg-slate-600 text-white rounded-md hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-slate-500 shadow-sm transition"
            >
              添加 API Key
            </button>
          )}
        </div>
        {showApiKeyForm && (
          <form onSubmit={handleAddApiKey} className="space-y-4 max-w-md mb-6 p-4 rounded-xl border border-slate-200 bg-gradient-to-br from-slate-50/50 to-white shadow-sm">
            <div className="space-y-2">
              <label htmlFor="api-key-name" className="block text-sm font-medium text-gray-700">名称</label>
              <input
                id="api-key-name"
                value={apiKeyName}
                onChange={(e) => setApiKeyName(e.target.value)}
                placeholder="例如：工作账号"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-slate-500"
              />
            </div>
            <div className="space-y-2">
              <label htmlFor="api-key-value" className="block text-sm font-medium text-gray-700">API Key</label>
              <input
                id="api-key-value"
                value={apiKeyValue}
                onChange={(e) => setApiKeyValue(e.target.value)}
                placeholder="zt_xxx"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-slate-500"
              />
              <p className="text-sm text-gray-500">
                请到 <a href="https://cloud.zectrix.com/home/api-keys" target="_blank" rel="noopener noreferrer" className="text-slate-600 hover:underline">https://cloud.zectrix.com/home/api-keys</a> 创建 API Key
              </p>
            </div>
            <div className="flex gap-2">
              <button
                type="submit"
                className="px-4 py-2 bg-slate-600 text-white rounded-md hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-slate-500 shadow-sm transition"
              >
                保存 API Key
              </button>
              {apiKeys.length > 0 && (
                <button
                  type="button"
                  onClick={() => setShowApiKeyForm(false)}
                  className="px-4 py-2 bg-gray-100 text-gray-600 rounded-md hover:bg-gray-200 transition"
                >
                  取消
                </button>
              )}
            </div>
          </form>
        )}

        <ul className="mt-6 space-y-2 max-w-[648px]">
          {apiKeys.map((key) => (
            <li key={key.id} className="flex items-center gap-3 p-3 border border-slate-200 rounded-md bg-gradient-to-br from-slate-50/30 to-white hover:shadow-sm transition">
              <div className="flex items-center gap-2 flex-1">
                <div className="w-1.5 h-1.5 rounded-full bg-slate-400"></div>
                <span className="font-medium text-gray-900">{key.name}</span>
                <span className="text-gray-400 text-sm ml-4 font-mono">{maskApiKey(key.key)}</span>
              </div>
              <button
                onClick={() => handleRemoveApiKey(key.id)}
                className="px-3 py-1 text-sm text-red-600 hover:text-red-700 hover:bg-red-50 rounded transition"
              >
                删除
              </button>
            </li>
          ))}
        </ul>
        {apiKeyError && <p role="alert" className="text-red-600 text-sm mt-2">{apiKeyError}</p>}
      </div>

      <div>
        <div className="flex items-center justify-between gap-3 mb-4">
          <div>
            <h3 className="text-lg font-semibold text-gray-900">设备管理</h3>
            <p className="text-sm text-gray-500">管理已连接的墨水屏设备。</p>
          </div>
          {!showDeviceForm && apiKeys.length > 0 && (
            <button
              type="button"
              onClick={() => setShowDeviceForm(true)}
              className="px-4 py-2 bg-emerald-500 text-white rounded-md hover:bg-emerald-600 focus:outline-none focus:ring-2 focus:ring-emerald-500 shadow-sm transition"
            >
              添加设备
            </button>
          )}
        </div>
        {apiKeys.length === 0 ? (
          <div className="rounded-lg border border-dashed border-slate-300 bg-slate-50/30 px-4 py-4 text-sm text-slate-500">
            请先添加 API Key
          </div>
        ) : showDeviceForm ? (
          <div className="space-y-4 max-w-md p-4 rounded-xl border border-emerald-200 bg-gradient-to-br from-emerald-50/30 to-white shadow-sm">
            <div className="space-y-2">
              <label id="api-key-select-label" htmlFor="api-key-select" className="block text-sm font-medium text-gray-700">选择 API Key</label>
              <Select value={selectedApiKey} onValueChange={setSelectedApiKey}>
                <SelectTrigger
                id="api-key-select"
                  aria-labelledby="api-key-select-label api-key-select"
                >
                  <SelectValue placeholder="请选择" />
                </SelectTrigger>
                <SelectContent>
                  {apiKeys.map((k) => (
                    <SelectItem key={k.id} value={String(k.id)}>{k.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <label htmlFor="device-id" className="block text-sm font-medium text-gray-700">MAC 地址</label>
              <input
                id="device-id"
                aria-label="MAC 地址"
                value={deviceId}
                onChange={(e) => setDeviceId(e.currentTarget.value)}
                placeholder="XX:XX:XX:XX:XX:XX"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-emerald-500"
              />
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleAddDevice}
                className="px-4 py-2 bg-emerald-500 text-white rounded-md hover:bg-emerald-600 focus:outline-none focus:ring-2 focus:ring-emerald-500 shadow-sm transition"
              >
                保存设备
              </button>
              {devices.length > 0 && (
                <button
                  type="button"
                  onClick={() => setShowDeviceForm(false)}
                  className="px-4 py-2 bg-gray-100 text-gray-600 rounded-md hover:bg-gray-200 transition"
                >
                  取消
                </button>
              )}
            </div>
            {deviceError && <p role="alert" className="text-red-600 text-sm">{deviceError}</p>}
          </div>
        ) : (
          deviceError ? <p role="alert" className="text-red-600 text-sm">{deviceError}</p> : null
        )}

        <ul className="mt-6 space-y-2 max-w-[648px]">
          {devices.map((device) => (
            <li key={device.deviceId} className="flex items-center gap-3 p-3 border border-emerald-200 rounded-md bg-gradient-to-br from-emerald-50/30 to-white hover:shadow-sm transition">
              <div className="flex items-center gap-2 flex-1">
                <div className="w-1.5 h-1.5 rounded-full bg-emerald-400"></div>
                <span className="font-medium text-gray-900">{device.alias}</span>
                <span className="text-gray-500 text-sm ml-4">{device.board}</span>
                <span className="text-xs text-emerald-600 bg-emerald-100 px-2 py-0.5 rounded ml-4">{getApiKeyName(device.apiKeyId)}</span>
              </div>
              <button
                onClick={() => handleRemoveDevice(device.deviceId)}
                className="px-3 py-1 text-sm text-red-600 hover:text-red-700 hover:bg-red-50 rounded transition"
              >
                删除
              </button>
            </li>
          ))}
        </ul>
      </div>

      {/* 关于 / 更新检测 */}
      <div className="mt-8">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">关于</h3>
        <div className="p-4 rounded-xl border border-gray-200 bg-gradient-to-br from-gray-50/50 to-white shadow-sm">
          <div className="flex items-center gap-4 mb-3">
            <div className="text-sm">
              <span className="text-gray-500">当前版本：</span>
              <span className="font-mono text-gray-900">{currentVersion || "加载中..."}</span>
            </div>
            <button
              onClick={handleCheckUpdate}
              disabled={checkingUpdate}
              className="px-4 py-2 bg-slate-600 text-white text-sm rounded-md hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm transition"
            >
              {checkingUpdate ? "检查中..." : "检查更新"}
            </button>
          </div>
          {updateError && (
            <p role="alert" className="text-red-600 text-sm">{updateError}</p>
          )}
          {updateInfo && (
            <div className="mt-3 p-3 border border-gray-200 rounded-md bg-white">
              {updateInfo.has_update ? (
                <div className="flex items-center gap-3">
                  <span className="text-emerald-600 font-medium flex items-center gap-1">
                    <div className="w-1.5 h-1.5 rounded-full bg-emerald-500"></div>
                    发现新版本 v{updateInfo.latest_version}
                  </span>
                  <button
                    onClick={handleOpenRelease}
                    className="px-3 py-1 text-sm text-blue-600 hover:text-blue-700 underline"
                  >
                    前往 GitHub Release 页面下载
                  </button>
                </div>
              ) : (
                <span className="text-gray-500 flex items-center gap-1">
                  <div className="w-1.5 h-1.5 rounded-full bg-gray-400"></div>
                  已是最新版本
                </span>
              )}
            </div>
          )}
        </div>
      </div>

        <div>
          <div className="mb-4">
            <h3 className="text-lg font-semibold text-gray-900">日历同步</h3>
            <p className="text-sm text-gray-500">将待办同步到 macOS 日历事件或提醒事项。</p>
          </div>
          <CalendarSyncPanel />
        </div>
      </div>

      <footer className="pt-4 pb-2 flex justify-center items-center gap-2 text-gray-500 text-sm">
        <HeartIcon size={16} />
        Made by
        <a
          href="https://space.bilibili.com/328381287"
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center gap-1.5 hover:text-gray-700 transition-colors"
        >
          <BilibiliIcon size={16} />
          <span>Terminator-AI</span>
        </a>
      </footer>
    </section>
  );
}
