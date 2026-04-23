import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/react";
import { TodoListPage } from "./todo-list-page";

const mockTodo = {
  localId: "local-1",
  id: null,
  title: "买牛奶",
  description: "低脂",
  dueDate: null,
  dueTime: null,
  status: 0,
  priority: 1,
  deviceId: "AA:BB:CC:DD:EE:FF",
  dirty: true,
  deleted: false,
  createdAt: "2026-04-23T10:00:00Z",
  updatedAt: "2026-04-23T10:00:00Z",
};

test("creates a todo from the collapsible form and marks it as local", async () => {
  const user = userEvent.setup();
  const createTodo = vi.fn().mockResolvedValue(mockTodo);

  render(
    <TodoListPage
      todos={[]}
      devices={[{ deviceId: "AA:BB:CC:DD:EE:FF", alias: "Desk", board: "bread-compact-wifi" }]}
      onCreateTodo={createTodo}
      onToggleTodo={vi.fn()}
      onDeleteTodo={vi.fn()}
      onUpdateTodo={vi.fn()}
      onPushTodo={vi.fn()}
    />,
  );

  await user.click(screen.getByRole("button", { name: "添加待办" }));
  await user.type(screen.getByLabelText("标题"), "买牛奶");
  await user.type(screen.getByLabelText("描述"), "低脂");
  await user.click(screen.getByRole("button", { name: "保存待办" }));

  expect(createTodo).toHaveBeenCalledWith({
    title: "买牛奶",
    description: "低脂",
    dueDate: undefined,
    dueTime: undefined,
    priority: 1,
    deviceId: undefined,
  });
  expect(await screen.findByText("买牛奶")).toBeInTheDocument();
  expect(screen.getByText("本地")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "推送" })).toBeInTheDocument();
});

test("creates a todo with the selected device from the shared ui select", async () => {
  const user = userEvent.setup();
  const createTodo = vi.fn().mockResolvedValue(mockTodo);

  render(
    <TodoListPage
      todos={[]}
      devices={[{ deviceId: "AA:BB:CC:DD:EE:FF", alias: "Desk", board: "bread-compact-wifi" }]}
      onCreateTodo={createTodo}
      onToggleTodo={vi.fn()}
      onDeleteTodo={vi.fn()}
      onUpdateTodo={vi.fn()}
      onPushTodo={vi.fn()}
    />,
  );

  await user.click(screen.getByRole("button", { name: "添加待办" }));
  await user.type(screen.getByLabelText("标题"), "买牛奶");
  await user.click(screen.getByRole("combobox", { name: "设备" }));
  await user.click(screen.getByRole("option", { name: "Desk" }));
  await user.click(screen.getByRole("button", { name: "保存待办" }));

  expect(createTodo).toHaveBeenCalledWith({
    title: "买牛奶",
    description: "",
    dueDate: undefined,
    dueTime: undefined,
    priority: 1,
    deviceId: "AA:BB:CC:DD:EE:FF",
  });
});

test("renders local and remote todo badges", async () => {
  render(
    <TodoListPage
      todos={[
        {
          localId: "local-1",
          id: null,
          title: "本地待办",
          description: "",
          dueDate: null,
          dueTime: null,
          status: 0,
          priority: 1,
          deviceId: null,
          dirty: true,
          deleted: false,
          createdAt: "2026-04-23T10:00:00Z",
          updatedAt: "2026-04-23T10:00:00Z",
        },
        {
          localId: "local-2",
          id: 9,
          title: "云端待办",
          description: "",
          dueDate: null,
          dueTime: null,
          status: 0,
          priority: 1,
          deviceId: null,
          dirty: false,
          deleted: false,
          createdAt: "2026-04-23T10:00:00Z",
          updatedAt: "2026-04-23T10:00:00Z",
        },
      ]}
      devices={[]}
      onCreateTodo={vi.fn()}
      onToggleTodo={vi.fn()}
      onDeleteTodo={vi.fn()}
      onUpdateTodo={vi.fn()}
      onPushTodo={vi.fn()}
    />,
  );

  expect(screen.getByText("本地")).toBeInTheDocument();
  expect(screen.getByText("云端")).toBeInTheDocument();
});

test("badge reflects remote identity rather than dirty state", async () => {
  render(
    <TodoListPage
      todos={[
        {
          localId: "local-1",
          id: null,
          title: "本地未同步",
          description: "",
          dueDate: null,
          dueTime: null,
          status: 0,
          priority: 1,
          deviceId: null,
          dirty: false,
          deleted: false,
          createdAt: "2026-04-23T10:00:00Z",
          updatedAt: "2026-04-23T10:00:00Z",
        },
        {
          localId: "local-2",
          id: 10,
          title: "云端已同步",
          description: "",
          dueDate: null,
          dueTime: null,
          status: 0,
          priority: 1,
          deviceId: null,
          dirty: true,
          deleted: false,
          createdAt: "2026-04-23T10:00:00Z",
          updatedAt: "2026-04-23T10:00:00Z",
        },
      ]}
      devices={[]}
      onCreateTodo={vi.fn()}
      onToggleTodo={vi.fn()}
      onDeleteTodo={vi.fn()}
      onUpdateTodo={vi.fn()}
      onPushTodo={vi.fn()}
    />,
  );

  expect(screen.getByText("本地未同步").parentElement?.querySelector(".bg-amber-100")).toHaveTextContent("本地");
  expect(screen.getByText("云端已同步").parentElement?.querySelector(".bg-emerald-100")).toHaveTextContent("云端");
});

test("pushing a local todo does not flip it into remote ui state", async () => {
  const user = userEvent.setup();

  render(
    <TodoListPage
      todos={[
        {
          localId: "local-1",
          id: null,
          title: "待推送",
          description: "",
          dueDate: null,
          dueTime: null,
          status: 0,
          priority: 1,
          deviceId: null,
          dirty: true,
          deleted: false,
          createdAt: "2026-04-23T10:00:00Z",
          updatedAt: "2026-04-23T10:00:00Z",
        },
      ]}
      devices={[{ deviceId: "AA:BB:CC:DD:EE:FF", alias: "Desk", board: "bread-compact-wifi" }]}
      onCreateTodo={vi.fn()}
      onToggleTodo={vi.fn()}
      onDeleteTodo={vi.fn()}
      onUpdateTodo={vi.fn()}
      onPushTodo={vi.fn().mockResolvedValue(undefined)}
    />,
  );

  await user.click(screen.getByRole("button", { name: "推送" }));

  expect(screen.getByText("本地")).toBeInTheDocument();
});

test("toggle callback receives localId", async () => {
  const user = userEvent.setup();
  const toggleTodo = vi.fn().mockResolvedValue({ ...mockTodo, status: 1 });

  render(
    <TodoListPage
      todos={[mockTodo]}
      devices={[]}
      onCreateTodo={vi.fn()}
      onToggleTodo={toggleTodo}
      onDeleteTodo={vi.fn()}
      onUpdateTodo={vi.fn()}
      onPushTodo={vi.fn()}
    />,
  );

  await user.click(screen.getByRole("checkbox", { name: "完成 买牛奶" }));

  expect(toggleTodo).toHaveBeenCalledWith("local-1");
});

test("push callback receives localId and target device", async () => {
  const user = userEvent.setup();
  const pushTodo = vi.fn().mockResolvedValue(undefined);

  render(
    <TodoListPage
      todos={[{ ...mockTodo, deviceId: null }]}
      devices={[{ deviceId: "AA:BB:CC:DD:EE:FF", alias: "Desk", board: "bread-compact-wifi" }]}
      onCreateTodo={vi.fn()}
      onToggleTodo={vi.fn()}
      onDeleteTodo={vi.fn()}
      onUpdateTodo={vi.fn()}
      onPushTodo={pushTodo}
    />,
  );

  await user.click(screen.getByRole("button", { name: "推送" }));

  expect(pushTodo).toHaveBeenCalledWith("local-1", "AA:BB:CC:DD:EE:FF");
});
