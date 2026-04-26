import { useEffect, useState } from "react";
import type { TodoRecord, TodoUpsertInput } from "../../lib/tauri";
import type { SyncState } from "../sync/sync-status";
import { toast } from "../../components/ui/toast";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "../../components/ui/dialog";
import { useAnalytics } from "../../hooks/useAnalytics";

type Device = { deviceId: string; alias: string; board: string };

const REPEAT_TYPE_OPTIONS = [
  { value: "none", label: "不重复" },
  { value: "daily", label: "每天" },
  { value: "weekly", label: "每周" },
  { value: "monthly", label: "每月" },
  { value: "yearly", label: "每年" },
];

const WEEKDAY_OPTIONS = [
  { value: "0", label: "周日" },
  { value: "1", label: "周一" },
  { value: "2", label: "周二" },
  { value: "3", label: "周三" },
  { value: "4", label: "周四" },
  { value: "5", label: "周五" },
  { value: "6", label: "周六" },
];

const PRIORITY_OPTIONS = [
  { value: "0", label: "普通" },
  { value: "1", label: "重要" },
  { value: "2", label: "紧急" },
];

type Props = {
  todos: TodoRecord[];
  devices: Device[];
  syncState?: SyncState;
  onSync?: () => void;
  onCreateTodo: (input: TodoUpsertInput) => Promise<TodoRecord>;
  onToggleTodo: (localId: string) => Promise<TodoRecord>;
  onDeleteTodo: (localId: string) => Promise<void>;
  onUpdateTodo: (localId: string, input: TodoUpsertInput) => Promise<TodoRecord>;
  onPushTodo: (localId: string, deviceId: string) => Promise<void>;
};

function formatDeadline(todo: TodoRecord) {
  if (!todo.dueDate && !todo.dueTime) {
    return "未设置截止时间";
  }
  if (todo.dueDate && todo.dueTime) {
    return `${todo.dueDate} ${todo.dueTime}`;
  }
  return todo.dueDate ?? todo.dueTime ?? "未设置截止时间";
}

export function TodoListPage({
  todos: initialTodos,
  devices,
  syncState = "idle",
  onSync,
  onCreateTodo,
  onToggleTodo,
  onDeleteTodo,
  onUpdateTodo,
  onPushTodo,
}: Props) {
  const { trackTodoCreate, trackTodoComplete, trackTodoDelete, trackPushText } = useAnalytics();
  const [todos, setTodos] = useState<TodoRecord[]>(initialTodos);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [dueDate, setDueDate] = useState("");
  const [dueTime, setDueTime] = useState("");
  const [dueDateActive, setDueDateActive] = useState(false);
  const [dueTimeActive, setDueTimeActive] = useState(false);
  const [deviceId, setDeviceId] = useState("");
  const [repeatType, setRepeatType] = useState("none");
  const [repeatWeekday, setRepeatWeekday] = useState("");
  const [repeatMonth, setRepeatMonth] = useState("");
  const [repeatDay, setRepeatDay] = useState("");
  const [priority, setPriority] = useState("0");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [pushingId, setPushingId] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState(false);
  const [editingTodo, setEditingTodo] = useState<TodoRecord | null>(null);

  // 同步外部 todos 变化到内部 state
  useEffect(() => {
    setTodos(initialTodos.filter((t) => !t.deleted));
  }, [initialTodos]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setActionLoading(true);
    try {
      const input: TodoUpsertInput = {
        title,
        description,
        dueDate: dueDate || undefined,
        dueTime: dueTime || undefined,
        repeatType: repeatType === "none" ? undefined : repeatType,
        repeatWeekday: repeatType === "weekly" && repeatWeekday ? Number(repeatWeekday) : undefined,
        repeatMonth: repeatType === "yearly" && repeatMonth ? Number(repeatMonth) : undefined,
        repeatDay: (repeatType === "monthly" || repeatType === "yearly") && repeatDay ? Number(repeatDay) : undefined,
        priority: Number(priority),
        deviceId: deviceId || undefined,
      };
      if (editingTodo) {
        const updated = await onUpdateTodo(editingTodo.localId, input);
        setTodos((prev) => prev.map((t) => (t.localId === updated.localId ? updated : t)));
        setEditingTodo(null);
      } else {
        const created = await onCreateTodo(input);
        trackTodoCreate(created.localId);
        setTodos((prev) => [created, ...prev].filter((t) => !t.deleted));
      }
      resetForm();
      setDialogOpen(false);
    } finally {
      setActionLoading(false);
    }
  }

  async function handleToggle(localId: string) {
    const updated = await onToggleTodo(localId);
    if (updated.status === 1) {
      trackTodoComplete(updated.localId);
    }
    setTodos((prev) => prev.map((t) => (t.localId === updated.localId ? updated : t)));
  }

  async function handleDelete(localId: string) {
    await onDeleteTodo(localId);
    trackTodoDelete(localId);
    setTodos((prev) => prev.filter((t) => t.localId !== localId && !t.deleted));
  }

  function handleEdit(todo: TodoRecord) {
    setEditingTodo(todo);
    setTitle(todo.title);
    setDescription(todo.description);
    setDueDate(todo.dueDate ?? "");
    setDueTime(todo.dueTime ?? "");
    setDeviceId(todo.deviceId ?? "");
    setRepeatType(todo.repeatType ?? "none");
    setRepeatWeekday(todo.repeatWeekday?.toString() ?? "");
    setRepeatMonth(todo.repeatMonth?.toString() ?? "");
    setRepeatDay(todo.repeatDay?.toString() ?? "");
    setPriority(todo.priority?.toString() ?? "0");
    setDueDateActive(Boolean(todo.dueDate));
    setDueTimeActive(Boolean(todo.dueTime));
    setDialogOpen(true);
  }

  async function handlePush(localId: string) {
    const todo = todos.find((t) => t.localId === localId);
    const target = todo?.deviceId ?? devices[0]?.deviceId;
    if (!target) {
      toast.error("没有可用的设备，请在设置中添加设备");
      return;
    }
    setPushingId(localId);
    try {
      await onPushTodo(localId, target);
      trackPushText(target);
      toast.success("待办推送成功");
    } catch (e) {
      toast.error(`推送失败: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setPushingId(null);
    }
  }

  function resetForm() {
    setTitle("");
    setDescription("");
    setDueDate("");
    setDueTime("");
    setDueDateActive(false);
    setDueTimeActive(false);
    setDeviceId("");
    setRepeatType("none");
    setRepeatWeekday("");
    setRepeatMonth("");
    setRepeatDay("");
    setPriority("0");
    setEditingTodo(null);
  }

  const visibleTodos = todos.filter((t) => !t.deleted);

  return (
    <section className="space-y-6">
      <header className="rounded-lg bg-gradient-to-r from-blue-50 to-sky-50 px-4 py-3 border border-blue-100">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">待办列表</h2>
            <p className="text-sm text-gray-500">管理你的待办事项，支持推送到墨水屏设备。</p>
          </div>
          <div className="flex items-center gap-3">
            {onSync && (
              <button
                type="button"
                disabled={syncState === "syncing"}
                onClick={onSync}
                className="px-4 py-2 border border-blue-300 text-blue-600 rounded-md hover:bg-blue-50 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
              >
                {syncState === "syncing" ? "同步中..." : "同步待办"}
              </button>
            )}
            <Dialog open={dialogOpen} onOpenChange={(open) => { setDialogOpen(open); if (!open) resetForm(); }}>
              <DialogTrigger asChild>
                <button
                  type="button"
                  className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 shadow-sm"
                >
                  添加待办
                </button>
              </DialogTrigger>
              <DialogContent>
            <DialogHeader>
              <DialogTitle>{editingTodo ? "编辑待办" : "添加待办"}</DialogTitle>
            </DialogHeader>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <label htmlFor="todo-title" className="block text-sm font-medium">标题</label>
                <input
                  id="todo-title"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  required
                />
              </div>
              <div className="space-y-2">
                <label htmlFor="todo-desc" className="block text-sm font-medium">描述</label>
                <textarea
                  id="todo-desc"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <label htmlFor="todo-due-date" className="block text-sm font-medium">截止日期</label>
                  {dueDateActive || dueDate ? (
                    <div className="relative">
                      <input
                        id="todo-due-date"
                        type="date"
                        value={dueDate}
                        onChange={(e) => setDueDate(e.target.value)}
                        onBlur={() => { if (!dueDate) setDueDateActive(false); }}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      />
                      {dueDate && (
                        <button
                          type="button"
                          onClick={() => { setDueDate(""); setDueDateActive(false); }}
                          className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
                          aria-label="清除截止日期"
                        >
                          &#x2715;
                        </button>
                      )}
                    </div>
                  ) : (
                    <input
                      id="todo-due-date"
                      type="text"
                      placeholder="不设置截止日期"
                      onFocus={() => setDueDateActive(true)}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-400"
                    />
                  )}
                </div>
                <div className="space-y-2">
                  <label htmlFor="todo-due-time" className="block text-sm font-medium">截止时间</label>
                  {dueTimeActive || dueTime ? (
                    <div className="relative">
                      <input
                        id="todo-due-time"
                        type="time"
                        value={dueTime}
                        onChange={(e) => setDueTime(e.target.value)}
                        onBlur={() => { if (!dueTime) setDueTimeActive(false); }}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      />
                      {dueTime && (
                        <button
                          type="button"
                          onClick={() => { setDueTime(""); setDueTimeActive(false); }}
                          className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
                          aria-label="清除截止时间"
                        >
                          &#x2715;
                        </button>
                      )}
                    </div>
                  ) : (
                    <input
                      id="todo-due-time"
                      type="text"
                      placeholder="不设置截止时间"
                      onFocus={() => setDueTimeActive(true)}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-400"
                    />
                  )}
                </div>
              </div>
              <div className="space-y-2">
                <label htmlFor="todo-priority" className="block text-sm font-medium">优先级</label>
                <Select value={priority} onValueChange={setPriority}>
                  <SelectTrigger id="todo-priority">
                    <SelectValue placeholder="普通" />
                  </SelectTrigger>
                  <SelectContent>
                    {PRIORITY_OPTIONS.map((opt) => (
                      <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <label htmlFor="todo-repeat-type" className="block text-sm font-medium">重复</label>
                  <Select value={repeatType} onValueChange={(v) => { setRepeatType(v); if (v === "none") { setRepeatWeekday(""); setRepeatMonth(""); setRepeatDay(""); } }}>
                    <SelectTrigger id="todo-repeat-type">
                      <SelectValue placeholder="不重复" />
                    </SelectTrigger>
                    <SelectContent>
                      {REPEAT_TYPE_OPTIONS.map((opt) => (
                        <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <label htmlFor="todo-repeat-period" className="block text-sm font-medium">重复周期</label>
                  {repeatType === "weekly" ? (
                    <Select value={repeatWeekday} onValueChange={setRepeatWeekday}>
                      <SelectTrigger id="todo-repeat-period">
                        <SelectValue placeholder="选择周几" />
                      </SelectTrigger>
                      <SelectContent>
                        {WEEKDAY_OPTIONS.map((opt) => (
                          <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  ) : repeatType === "yearly" ? (
                    <div className="flex gap-2">
                      <input
                        id="todo-repeat-month"
                        type="number"
                        min="1"
                        max="12"
                        value={repeatMonth}
                        onChange={(e) => setRepeatMonth(e.target.value)}
                        placeholder="月"
                        className="w-1/2 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      />
                      <input
                        id="todo-repeat-day"
                        type="number"
                        min="1"
                        max="31"
                        value={repeatDay}
                        onChange={(e) => setRepeatDay(e.target.value)}
                        placeholder="号"
                        className="w-1/2 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      />
                    </div>
                  ) : (
                    <input
                      id="todo-repeat-period"
                      type="number"
                      min="1"
                      max="31"
                      value={repeatDay}
                      onChange={(e) => setRepeatDay(e.target.value)}
                      placeholder={repeatType === "monthly" ? "每月几号 (1-31)" : "选择重复类型后设置"}
                      disabled={repeatType === "none"}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
                    />
                  )}
                </div>
              </div>
              {devices.length > 0 && (
                <div className="space-y-2">
                  <label htmlFor="todo-device" className="block text-sm font-medium">设备</label>
                  <Select value={deviceId} onValueChange={setDeviceId}>
                    <SelectTrigger id="todo-device">
                      <SelectValue placeholder="不指定" />
                    </SelectTrigger>
                    <SelectContent>
                      {devices.map((d) => (
                        <SelectItem key={d.deviceId} value={d.deviceId}>{d.alias}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}
              <div className="flex gap-2">
                <button
                  type="submit"
                  disabled={actionLoading}
                  className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
                >
                  {actionLoading ? "保存中..." : editingTodo ? "保存修改" : "保存待办"}
                </button>
              </div>
            </form>
          </DialogContent>
        </Dialog>
          </div>
        </div>
      </header>

      <ul className="space-y-3">
        {visibleTodos.map((todo) => (
          <li key={todo.localId} className="rounded-xl border border-blue-200 bg-gradient-to-br from-blue-50/30 to-white p-4 shadow-sm hover:shadow-md transition">
            <div className="flex items-start gap-3">
              <input
                type="checkbox"
                checked={todo.status === 1}
                onChange={() => handleToggle(todo.localId)}
                aria-label={`完成 ${todo.title}`}
                className="mt-1 h-4 w-4 accent-blue-500"
              />
              <div className="min-w-0 flex-1 space-y-1">
                <div className="flex flex-wrap items-center gap-2">
                  <div className="flex items-center gap-2">
                    <div className={`w-1.5 h-1.5 rounded-full ${todo.status === 1 ? "bg-gray-300" : todo.priority === 2 ? "bg-red-500" : todo.priority === 1 ? "bg-amber-500" : "bg-blue-400"}`}></div>
                    <span className={todo.status === 1 ? "text-gray-500 line-through font-medium" : "font-medium text-gray-900"}>{todo.title}</span>
                  </div>
                  <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${todo.id === null ? "bg-amber-100 text-amber-700" : "bg-emerald-100 text-emerald-700"}`}>
                    {todo.id === null ? "本地" : "云端"}
                  </span>
                </div>
                {todo.description ? <p className="text-sm text-gray-600 pl-3.5">{todo.description}</p> : null}
                <p className="text-sm text-gray-500 pl-3.5">{formatDeadline(todo)}</p>
              </div>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => handleEdit(todo)}
                  className="px-2 py-1 text-sm text-gray-600 hover:text-blue-600 transition"
                  aria-label={`编辑 ${todo.title}`}
                >
                  编辑
                </button>
                <button
                  type="button"
                  onClick={() => handleDelete(todo.localId)}
                  className="px-2 py-1 text-sm text-gray-600 hover:text-red-600 transition"
                  aria-label={`删除 ${todo.title}`}
                >
                  删除
                </button>
                <button
                  type="button"
                  onClick={() => handlePush(todo.localId)}
                  disabled={pushingId === todo.localId}
                  className="px-3 py-1 text-sm bg-blue-500 text-white rounded-md hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm transition"
                >
                  {pushingId === todo.localId ? "推送中..." : "推送"}
                </button>
              </div>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
