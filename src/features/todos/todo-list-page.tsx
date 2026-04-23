import { useEffect, useState } from "react";
import type { TodoRecord, TodoUpsertInput } from "../../lib/tauri";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "../../components/ui/dialog";

type Device = { deviceId: string; alias: string; board: string };

type Props = {
  todos: TodoRecord[];
  devices: Device[];
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
  onCreateTodo,
  onToggleTodo,
  onDeleteTodo,
  onUpdateTodo,
  onPushTodo,
}: Props) {
  const [todos, setTodos] = useState<TodoRecord[]>(initialTodos);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [dueDate, setDueDate] = useState("");
  const [dueTime, setDueTime] = useState("");
  const [dueDateActive, setDueDateActive] = useState(false);
  const [dueTimeActive, setDueTimeActive] = useState(false);
  const [deviceId, setDeviceId] = useState("");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [pushError, setPushError] = useState<string | null>(null);
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
      if (editingTodo) {
        const updated = await onUpdateTodo(editingTodo.localId, {
          title,
          description,
          dueDate: dueDate || undefined,
          dueTime: dueTime || undefined,
          priority: editingTodo.priority,
          deviceId: deviceId || undefined,
        });
        setTodos((prev) => prev.map((t) => (t.localId === updated.localId ? updated : t)));
        setEditingTodo(null);
      } else {
        const created = await onCreateTodo({
          title,
          description,
          dueDate: dueDate || undefined,
          dueTime: dueTime || undefined,
          priority: 1,
          deviceId: deviceId || undefined,
        });
        setTodos((prev) => [created, ...prev].filter((t) => !t.deleted));
      }
      setTitle("");
      setDescription("");
      setDueDate("");
      setDueTime("");
      setDeviceId("");
      setDialogOpen(false);
    } finally {
      setActionLoading(false);
    }
  }

  async function handleToggle(localId: string) {
    const updated = await onToggleTodo(localId);
    setTodos((prev) => prev.map((t) => (t.localId === updated.localId ? updated : t)));
  }

  async function handleDelete(localId: string) {
    await onDeleteTodo(localId);
    setTodos((prev) => prev.filter((t) => t.localId !== localId && !t.deleted));
  }

  function handleEdit(todo: TodoRecord) {
    setEditingTodo(todo);
    setTitle(todo.title);
    setDescription(todo.description);
    setDueDate(todo.dueDate ?? "");
    setDueTime(todo.dueTime ?? "");
    setDeviceId(todo.deviceId ?? "");
    setDueDateActive(Boolean(todo.dueDate));
    setDueTimeActive(Boolean(todo.dueTime));
    setDialogOpen(true);
  }

  async function handlePush(localId: string) {
    const todo = todos.find((t) => t.localId === localId);
    const target = todo?.deviceId ?? devices[0]?.deviceId;
    if (!target) {
      setPushError("没有可用的设备，请在设置中添加设备");
      return;
    }
    setPushError(null);
    setPushingId(localId);
    try {
      await onPushTodo(localId, target);
    } catch (e) {
      setPushError(String(e));
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
    setEditingTodo(null);
  }

  const visibleTodos = todos.filter((t) => !t.deleted);

  return (
    <section className="space-y-6">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-lg font-semibold">待办列表</h2>
        <Dialog open={dialogOpen} onOpenChange={(open) => { setDialogOpen(open); if (!open) resetForm(); }}>
          <DialogTrigger asChild>
            <button
              type="button"
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
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

      <ul className="space-y-3">
        {pushError && (
          <li className="rounded-2xl border border-red-200 bg-red-50 p-4">
            <p role="alert" className="text-red-600 text-sm">{pushError}</p>
          </li>
        )}
        {visibleTodos.map((todo) => (
          <li key={todo.localId} className="rounded-2xl border border-gray-200 bg-white/85 p-4 shadow-sm">
            <div className="flex items-start gap-3">
              <input
                type="checkbox"
                checked={todo.status === 1}
                onChange={() => handleToggle(todo.localId)}
                aria-label={`完成 ${todo.title}`}
                className="mt-1 h-4 w-4"
              />
              <div className="min-w-0 flex-1 space-y-1">
                <div className="flex flex-wrap items-center gap-2">
                  <span className={todo.status === 1 ? "text-gray-500 line-through font-medium" : "font-medium"}>{todo.title}</span>
                  <span className={`rounded-full px-2 py-0.5 text-xs ${todo.id === null ? "bg-amber-100 text-amber-700" : "bg-emerald-100 text-emerald-700"}`}>
                    {todo.id === null ? "本地" : "云端"}
                  </span>
                </div>
                {todo.description ? <p className="text-sm text-gray-600">{todo.description}</p> : null}
                <p className="text-sm text-gray-500">{formatDeadline(todo)}</p>
              </div>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => handleEdit(todo)}
                  className="px-2 py-1 text-sm text-gray-600 hover:text-blue-600"
                  aria-label={`编辑 ${todo.title}`}
                >
                  编辑
                </button>
                <button
                  type="button"
                  onClick={() => handleDelete(todo.localId)}
                  className="px-2 py-1 text-sm text-gray-600 hover:text-red-600"
                  aria-label={`删除 ${todo.title}`}
                >
                  删除
                </button>
                <button
                  type="button"
                  onClick={() => handlePush(todo.localId)}
                  disabled={pushingId === todo.localId}
                  className="px-3 py-1 text-sm bg-gray-200 rounded hover:bg-gray-300 disabled:opacity-50 disabled:cursor-not-allowed"
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
