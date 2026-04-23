import { useState } from "react";
import type { DeviceRecord } from "../../lib/tauri";

type DeviceManagementPageProps = {
  devices: DeviceRecord[];
  onAddDevice: (deviceId: string) => Promise<DeviceRecord>;
  onRemoveDevice: (deviceId: string) => Promise<void>;
};

const macPattern = /^[0-9A-F]{2}(:[0-9A-F]{2}){5}$/;

export function DeviceManagementPage(props: DeviceManagementPageProps) {
  const [deviceId, setDeviceId] = useState("");
  const [devices, setDevices] = useState(props.devices);
  const [error, setError] = useState<string | null>(null);

  async function handleAdd() {
    const normalized = deviceId.trim().toUpperCase();
    if (!macPattern.test(normalized)) {
      setError("MAC 地址格式错误");
      return;
    }
    const device = await props.onAddDevice(normalized);
    setDevices((current) => [...current, device]);
    setDeviceId("");
    setError(null);
  }

  async function handleRemove(id: string) {
    await props.onRemoveDevice(id);
    setDevices((current) => current.filter((d) => d.deviceId !== id));
  }

  return (
    <section className="p-4">
      <h2 className="text-xl font-semibold mb-4">设备管理</h2>
      <div className="space-y-4 max-w-md">
        <div className="space-y-2">
          <label htmlFor="device-id" className="block text-sm font-medium">MAC 地址</label>
          <input
            id="device-id"
            aria-label="MAC 地址"
            value={deviceId}
            onChange={(e) => setDeviceId(e.currentTarget.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600"
          />
        </div>
        <button
          onClick={handleAdd}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          添加设备
        </button>
        {error && <p role="alert" className="text-red-600 text-sm">{error}</p>}
      </div>

      <ul className="mt-6 space-y-2">
        {devices.map((device) => (
          <li key={device.deviceId} className="flex items-center gap-3 p-2 border border-gray-200 rounded-md dark:border-gray-700">
            <span className="font-medium">{device.alias}</span>
            <span className="text-gray-500 text-sm">{device.board}</span>
            <button
              onClick={() => handleRemove(device.deviceId)}
              className="ml-auto px-3 py-1 text-sm text-red-600 hover:text-red-700"
            >
              删除
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}