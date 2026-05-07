import { useEffect, useState, useCallback } from "react";
import {
  type CalendarSyncConfig,
  type CalendarInfo,
  type SyncResult,
  getCalendarSyncConfig,
  saveCalendarSyncConfig,
  listCalendars,
  syncCalendar,
  requestCalendarPermission,
} from "../../lib/tauri";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";

const DEFAULT_CONFIG: CalendarSyncConfig = {
  enabled: false,
  direction: "toCalendar",
  targetType: "reminder",
  targetCalendarId: null,
};

export function CalendarSyncPanel() {
  const [config, setConfig] = useState<CalendarSyncConfig>(DEFAULT_CONFIG);
  const [calendars, setCalendars] = useState<CalendarInfo[]>([]);
  const [syncing, setSyncing] = useState(false);
  const [syncResult, setSyncResult] = useState<SyncResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [permissionDenied, setPermissionDenied] = useState(false);

  useEffect(() => {
    getCalendarSyncConfig()
      .then(setConfig)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    if (!config.enabled) return;
    listCalendars(config.targetType)
      .then(setCalendars)
      .catch((e) => {
        if (String(e).includes("permission")) {
          setPermissionDenied(true);
          setCalendars([]);
        } else {
          setError(String(e));
        }
      });
  }, [config.enabled, config.targetType]);

  const updateConfig = useCallback(
    async (patch: Partial<CalendarSyncConfig>) => {
      const next = { ...config, ...patch };
      setConfig(next);
      setSyncResult(null);
      try {
        await saveCalendarSyncConfig(next);
      } catch (e) {
        setError(String(e));
      }
    },
    [config]
  );

  async function handleSync() {
    setSyncing(true);
    setError(null);
    setSyncResult(null);
    try {
      const result = await syncCalendar();
      setSyncResult(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setSyncing(false);
    }
  }

  if (loading) return null;

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <label className="flex items-center gap-2 cursor-pointer select-none">
          <input
            type="checkbox"
            aria-label="启用日历同步"
            checked={config.enabled}
            onChange={(e) => updateConfig({ enabled: e.target.checked })}
            className="w-4 h-4 accent-slate-600"
          />
          <span className="text-sm font-medium text-gray-700">启用日历同步</span>
        </label>
      </div>

      {config.enabled && (
        <div className="space-y-4 pl-6 border-l-2 border-slate-200">
          {/* 目标类型 */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700">目标类型</label>
            <div className="flex gap-4">
              {(["reminder", "calendarEvent"] as const).map((t) => (
                <label key={t} className="flex items-center gap-1.5 cursor-pointer text-sm text-gray-700">
                  <input
                    type="radio"
                    name="targetType"
                    value={t}
                    checked={config.targetType === t}
                    onChange={() => updateConfig({ targetType: t, targetCalendarId: null })}
                    className="accent-slate-600"
                  />
                  {t === "reminder" ? "提醒事项" : "日历事件"}
                </label>
              ))}
            </div>
          </div>

          {/* 目标日历本 */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700">目标日历本</label>
            {permissionDenied ? (
              <div className="flex flex-col gap-2">
                <p className="text-sm text-red-600">尚未授权访问日历</p>
                <button
                  type="button"
                  onClick={async () => {
                    try {
                      const granted = await requestCalendarPermission();
                      setPermissionDenied(!granted);
                      if (granted) {
                        listCalendars(config.targetType)
                          .then(setCalendars)
                          .catch(() => setCalendars([]));
                      }
                    } catch (e) {
                      setError(String(e));
                    }
                  }}
                  className="px-4 py-2 bg-slate-600 text-white rounded-md hover:bg-slate-700 transition text-sm focus:outline-none focus:ring-2 focus:ring-slate-500 w-fit"
                >
                  请求权限
                </button>
              </div>
            ) : (
            <Select
              value={config.targetCalendarId ?? ""}
              onValueChange={(v) => updateConfig({ targetCalendarId: v || null })}
            >
              <SelectTrigger className="w-56">
                <SelectValue placeholder="请选择日历本" />
              </SelectTrigger>
              <SelectContent>
                {calendars.map((cal) => (
                  <SelectItem key={cal.id} value={cal.id}>
                    {cal.title}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            )}
          </div>

          {/* 同步方向 */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700">同步方向</label>
            <div className="flex flex-col gap-1.5">
              {(
                [
                  ["toCalendar", "仅推送到 Apple 日历"],
                  ["fromCalendar", "仅从 Apple 日历导入"],
                  ["bidirectional", "双向同步"],
                ] as const
              ).map(([val, label]) => (
                <label key={val} className="flex items-center gap-1.5 cursor-pointer text-sm text-gray-700">
                  <input
                    type="radio"
                    name="direction"
                    value={val}
                    checked={config.direction === val}
                    onChange={() => updateConfig({ direction: val })}
                    className="accent-slate-600"
                  />
                  {label}
                </label>
              ))}
            </div>
          </div>

          {/* 立即同步 */}
          <div className="space-y-2">
            <button
              type="button"
              onClick={handleSync}
              disabled={syncing || !config.targetCalendarId}
              className="px-4 py-2 bg-slate-600 text-white rounded-md hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed transition text-sm focus:outline-none focus:ring-2 focus:ring-slate-500"
            >
              {syncing ? "同步中…" : "立即同步"}
            </button>

            {syncResult && (
              <p className="text-sm text-green-700">
                新增 {syncResult.created} 条，更新 {syncResult.updated} 条，删除 {syncResult.deleted} 条，跳过 {syncResult.skipped} 条。
              </p>
            )}
            {error && (
              <p role="alert" className="text-sm text-red-600">
                {error}
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
