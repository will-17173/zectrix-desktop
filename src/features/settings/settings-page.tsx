import { FormEvent, useState } from "react";
import { type ApiKeyRecord, type DeviceRecord } from "../../lib/tauri";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";

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
  const [error, setError] = useState<string | null>(null);
  const [showApiKeyForm, setShowApiKeyForm] = useState(initialApiKeys.length === 0);
  const [showDeviceForm, setShowDeviceForm] = useState(initialDevices.length === 0);

  async function handleAddApiKey(event: FormEvent) {
    event.preventDefault();
    if (!apiKeyName.trim() || !apiKeyValue.trim()) {
      setError("请填写名称和 API Key");
      return;
    }
    try {
      const record = await onAddApiKey(apiKeyName.trim(), apiKeyValue.trim());
      setApiKeys((prev) => [...prev, record]);
      setApiKeyName("");
      setApiKeyValue("");
      setShowApiKeyForm(false);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleRemoveApiKey(id: number) {
    await onRemoveApiKey(id);
    setApiKeys((prev) => prev.filter((k) => k.id !== id));
  }

  async function handleAddDevice() {
    if (!selectedApiKey) {
      setError("请先选择 API Key");
      return;
    }
    const normalized = deviceId.trim().toUpperCase();
    if (!macPattern.test(normalized)) {
      setError("MAC 地址格式错误");
      return;
    }
    try {
      const device = await onAddDevice(normalized, Number(selectedApiKey));
      setDevices((prev) => [...prev, device]);
      setDeviceId("");
      setSelectedApiKey("");
      setShowDeviceForm(false);
      setError(null);
    } catch (e) {
      setError(String(e));
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
    <section className="p-4 space-y-8">
      <div>
        <div className="flex items-center justify-between gap-3 mb-4">
          <h2 className="text-xl font-semibold">API Key 管理</h2>
          {!showApiKeyForm && (
            <button
              type="button"
              onClick={() => setShowApiKeyForm(true)}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              添加 API Key
            </button>
          )}
        </div>
        {showApiKeyForm && (
          <form onSubmit={handleAddApiKey} className="space-y-4 max-w-md mb-6">
            <div className="space-y-2">
              <label htmlFor="api-key-name" className="block text-sm font-medium">名称</label>
              <input
                id="api-key-name"
                value={apiKeyName}
                onChange={(e) => setApiKeyName(e.target.value)}
                placeholder="例如：工作账号"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600"
              />
            </div>
            <div className="space-y-2">
              <label htmlFor="api-key-value" className="block text-sm font-medium">API Key</label>
              <input
                id="api-key-value"
                value={apiKeyValue}
                onChange={(e) => setApiKeyValue(e.target.value)}
                placeholder="zt_xxx"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600"
              />
              <p className="text-sm text-gray-500">
                请到 <a href="https://cloud.zectrix.com/home/api-keys" target="_blank" rel="noopener noreferrer" className="text-blue-600 hover:underline">https://cloud.zectrix.com/home/api-keys</a> 创建 API Key
              </p>
            </div>
            <div className="flex gap-2">
              <button
                type="submit"
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                保存 API Key
              </button>
              {apiKeys.length > 0 && (
                <button
                  type="button"
                  onClick={() => setShowApiKeyForm(false)}
                  className="px-4 py-2 bg-gray-200 rounded-md hover:bg-gray-300"
                >
                  取消
                </button>
              )}
            </div>
          </form>
        )}

        <ul className="mt-6 space-y-2 max-w-[648px]">
          {apiKeys.map((key) => (
            <li key={key.id} className="flex items-center gap-3 p-3 border border-gray-200 rounded-md dark:border-gray-700">
              <div className="flex-1">
                <span className="font-medium">{key.name}</span>
                <span className="text-gray-400 text-sm ml-6 font-mono">{maskApiKey(key.key)}</span>
              </div>
              <button
                onClick={() => handleRemoveApiKey(key.id)}
                className="px-3 py-1 text-sm text-red-600 hover:text-red-700"
              >
                删除
              </button>
            </li>
          ))}
        </ul>
      </div>

      <div>
        <div className="flex items-center justify-between gap-3 mb-4">
          <h2 className="text-xl font-semibold">设备管理</h2>
          {!showDeviceForm && apiKeys.length > 0 && (
            <button
              type="button"
              onClick={() => setShowDeviceForm(true)}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              添加设备
            </button>
          )}
        </div>
        {apiKeys.length === 0 ? (
          <p className="text-gray-500">请先添加 API Key</p>
        ) : showDeviceForm ? (
          <div className="space-y-4 max-w-md">
            <div className="space-y-2">
              <label id="api-key-select-label" htmlFor="api-key-select" className="block text-sm font-medium">选择 API Key</label>
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
              <label htmlFor="device-id" className="block text-sm font-medium">MAC 地址</label>
              <input
                id="device-id"
                aria-label="MAC 地址"
                value={deviceId}
                onChange={(e) => setDeviceId(e.currentTarget.value)}
                placeholder="XX:XX:XX:XX:XX:XX"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600"
              />
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleAddDevice}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                保存设备
              </button>
              {devices.length > 0 && (
                <button
                  type="button"
                  onClick={() => setShowDeviceForm(false)}
                  className="px-4 py-2 bg-gray-200 rounded-md hover:bg-gray-300"
                >
                  取消
                </button>
              )}
            </div>
            {error && <p role="alert" className="text-red-600 text-sm">{error}</p>}
          </div>
        ) : (
          error ? <p role="alert" className="text-red-600 text-sm">{error}</p> : null
        )}

        <ul className="mt-6 space-y-2 max-w-[648px]">
          {devices.map((device) => (
            <li key={device.deviceId} className="flex items-center gap-3 p-2 border border-gray-200 rounded-md dark:border-gray-700">
              <span className="font-medium">{device.alias}</span>
              <span className="text-gray-500 text-sm ml-6">{device.board}</span>
              <span className="text-xs text-blue-600 bg-blue-100 px-2 py-0.5 rounded ml-6">{getApiKeyName(device.apiKeyId)}</span>
              <button
                onClick={() => handleRemoveDevice(device.deviceId)}
                className="ml-auto px-3 py-1 text-sm text-red-600 hover:text-red-700"
              >
                删除
              </button>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
