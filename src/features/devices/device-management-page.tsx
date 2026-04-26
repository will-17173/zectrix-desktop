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
    <section className="p-4 space-y-6">
      <header className="rounded-lg bg-gradient-to-r from-emerald-50 to-teal-50 px-4 py-3 border border-emerald-100">
        <h2 className="text-xl font-semibold text-gray-900">设备管理</h2>
        <p className="text-sm text-gray-500">添加和管理你的墨水屏设备。</p>
      </header>
      <div className="space-y-4 max-w-md p-4 rounded-xl border border-emerald-200 bg-gradient-to-br from-emerald-50/30 to-white shadow-sm">
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
        <button
          onClick={handleAdd}
          className="px-4 py-2 bg-emerald-500 text-white rounded-md hover:bg-emerald-600 focus:outline-none focus:ring-2 focus:ring-emerald-500 shadow-sm transition"
        >
          添加设备
        </button>
        {error && <p role="alert" className="text-red-600 text-sm">{error}</p>}
      </div>

      <ul className="mt-6 space-y-2 max-w-md">
        {devices.map((device) => (
          <li key={device.deviceId} className="flex items-center gap-3 p-3 border border-emerald-200 rounded-md bg-gradient-to-br from-emerald-50/30 to-white shadow-sm hover:shadow-md transition">
            <div className="flex items-center gap-2 flex-1">
              <div className="w-1.5 h-1.5 rounded-full bg-emerald-400"></div>
              <span className="font-medium text-gray-900">{device.alias}</span>
              <span className="text-gray-500 text-sm ml-2">{device.board}</span>
            </div>
            <button
              onClick={() => handleRemove(device.deviceId)}
              className="px-3 py-1 text-sm text-red-600 hover:text-red-700 hover:bg-red-50 rounded transition"
            >
              删除
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}