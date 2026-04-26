import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";
import { PluginMarketPage } from "./plugin-market-page";

vi.mock("../../components/ui/toast", () => ({
  Toaster: () => null,
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    loading: vi.fn(),
  },
}));

const devices = [
  {
    deviceId: "AA:BB:CC:DD:EE:FF",
    alias: "Desk",
    board: "board",
    cachedAt: "2026-04-25T00:00:00Z",
    apiKeyId: 1,
  },
];

test("creates a custom plugin", async () => {
  const user = userEvent.setup();
  const save = vi.fn().mockResolvedValue({
    id: 1,
    name: "天气",
    description: "天气插件",
    code: "return { type: 'text', text: 'sunny' };",
    createdAt: "2026-04-25T00:00:00Z",
    updatedAt: "2026-04-25T00:00:00Z",
  });

  render(
    <PluginMarketPage
      devices={devices}
      builtinPlugins={[]}
      customPlugins={[]}
      pluginLoopTasks={[]}
      onSavePlugin={save}
      onDeletePlugin={vi.fn()}
      onPushPlugin={vi.fn()}
      onCreateLoopTask={vi.fn()}
      onDeleteLoopTask={vi.fn()}
      onStartLoopTask={vi.fn()}
      onStopLoopTask={vi.fn()}
    />,
  );

  await user.click(screen.getByRole("tab", { name: "自定义插件" }));
  await user.click(screen.getByRole("button", { name: "新增插件" }));
  await user.type(screen.getByLabelText("插件名称"), "天气");
  await user.type(screen.getByLabelText("插件描述"), "天气插件");
  await user.clear(screen.getByLabelText("插件代码"));
  await user.type(screen.getByLabelText("插件代码"), "return {{ type: 'text', text: 'sunny' }};");
  await user.click(screen.getByRole("button", { name: "保存插件" }));

  expect(save).toHaveBeenCalled();
});

test("prefills new custom plugins with a complete async function template", async () => {
  const user = userEvent.setup();

  render(
    <PluginMarketPage
      devices={devices}
      builtinPlugins={[]}
      customPlugins={[]}
      pluginLoopTasks={[]}
      onSavePlugin={vi.fn()}
      onDeletePlugin={vi.fn()}
      onPushPlugin={vi.fn()}
      onCreateLoopTask={vi.fn()}
      onDeleteLoopTask={vi.fn()}
      onStartLoopTask={vi.fn()}
      onStopLoopTask={vi.fn()}
    />,
  );

  await user.click(screen.getByRole("tab", { name: "自定义插件" }));
  await user.click(screen.getByRole("button", { name: "新增插件" }));

  const code = screen.getByLabelText("插件代码") as HTMLTextAreaElement;

  expect(code.value).toContain("(async function()");
  expect(code.value).toContain("// 用户代码开始");
  expect(code.value).toContain("// 用户代码结束");
  expect(code.value).toContain("})()");
  expect(code.value).not.toContain("textImage");
});

test("opens custom plugin usage instructions from the custom tab", async () => {
  const user = userEvent.setup();

  render(
    <PluginMarketPage
      devices={devices}
      builtinPlugins={[]}
      customPlugins={[]}
      pluginLoopTasks={[]}
      onSavePlugin={vi.fn()}
      onDeletePlugin={vi.fn()}
      onPushPlugin={vi.fn()}
      onCreateLoopTask={vi.fn()}
      onDeleteLoopTask={vi.fn()}
      onStartLoopTask={vi.fn()}
      onStopLoopTask={vi.fn()}
    />,
  );

  await user.click(screen.getByRole("tab", { name: "自定义插件" }));
  await user.click(screen.getByRole("button", { name: "使用方法" }));

  expect(screen.getByRole("dialog", { name: "自定义插件使用方法" })).toBeInTheDocument();
  expect(screen.getByText("插件代码会在异步函数中执行，可以返回文本或图片结果。")).toBeInTheDocument();
  expect(screen.getByText("返回格式")).toBeInTheDocument();
  expect(screen.getByText("文本示例")).toBeInTheDocument();
  expect(screen.getByText("图片 URL 示例")).toBeInTheDocument();
  expect(screen.getByText(/应用会先下载图片/)).toBeInTheDocument();
  expect(screen.getByText("循环任务注意事项")).toBeInTheDocument();
});

test("pushes a custom plugin to selected page", async () => {
  const user = userEvent.setup();
  const push = vi.fn().mockResolvedValue(undefined);

  render(
    <PluginMarketPage
      devices={devices}
      builtinPlugins={[]}
      customPlugins={[
        {
          id: 7,
          name: "天气",
          description: "天气插件",
          code: "return { type: 'text', text: 'sunny' };",
          createdAt: "2026-04-25T00:00:00Z",
          updatedAt: "2026-04-25T00:00:00Z",
        },
      ]}
      pluginLoopTasks={[]}
      onSavePlugin={vi.fn()}
      onDeletePlugin={vi.fn()}
      onPushPlugin={push}
      onCreateLoopTask={vi.fn()}
      onDeleteLoopTask={vi.fn()}
      onStartLoopTask={vi.fn()}
      onStopLoopTask={vi.fn()}
    />,
  );

  await user.click(screen.getByRole("tab", { name: "自定义插件" }));
  await user.click(screen.getByRole("button", { name: "推送一次" }));

  expect(push).toHaveBeenCalledWith("custom", "7", "AA:BB:CC:DD:EE:FF", 1);
});

test("edits a custom plugin only from the card edit button", async () => {
  const user = userEvent.setup();

  render(
    <PluginMarketPage
      devices={devices}
      builtinPlugins={[]}
      customPlugins={[
        {
          id: 7,
          name: "天气",
          description: "天气插件",
          code: "return { type: 'text', text: 'sunny' };",
          createdAt: "2026-04-25T00:00:00Z",
          updatedAt: "2026-04-25T00:00:00Z",
        },
      ]}
      pluginLoopTasks={[]}
      onSavePlugin={vi.fn()}
      onDeletePlugin={vi.fn()}
      onPushPlugin={vi.fn()}
      onCreateLoopTask={vi.fn()}
      onDeleteLoopTask={vi.fn()}
      onStartLoopTask={vi.fn()}
      onStopLoopTask={vi.fn()}
    />,
  );

  await user.click(screen.getByRole("tab", { name: "自定义插件" }));
  await user.click(screen.getByText("天气"));

  expect(screen.queryByRole("dialog", { name: "编辑插件 #7" })).not.toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "编辑" }));

  expect(screen.getByRole("dialog", { name: "编辑插件 #7" })).toBeInTheDocument();
});

test("creates a loop task for a custom plugin", async () => {
  const user = userEvent.setup();
  const createLoopTask = vi.fn().mockResolvedValue({
    id: 1,
    pluginKind: "custom",
    pluginId: "7",
    name: "天气循环",
    deviceId: "AA:BB:CC:DD:EE:FF",
    pageId: 1,
    intervalSeconds: 60,
    durationType: "none",
    status: "idle",
    createdAt: "2026-04-25T00:00:00Z",
    updatedAt: "2026-04-25T00:00:00Z",
  });

  render(
    <PluginMarketPage
      devices={devices}
      builtinPlugins={[]}
      customPlugins={[
        {
          id: 7,
          name: "天气",
          description: "天气插件",
          code: "return { type: 'text', text: 'sunny' };",
          createdAt: "2026-04-25T00:00:00Z",
          updatedAt: "2026-04-25T00:00:00Z",
        },
      ]}
      pluginLoopTasks={[]}
      onSavePlugin={vi.fn()}
      onDeletePlugin={vi.fn()}
      onPushPlugin={vi.fn()}
      onCreateLoopTask={createLoopTask}
      onDeleteLoopTask={vi.fn()}
      onStartLoopTask={vi.fn()}
      onStopLoopTask={vi.fn()}
    />,
  );

  await user.click(screen.getByRole("tab", { name: "自定义插件" }));
  await user.click(screen.getByRole("button", { name: "循环" }));

  expect(createLoopTask).toHaveBeenCalledWith(
    expect.objectContaining({
      pluginKind: "custom",
      pluginId: "7",
      deviceId: "AA:BB:CC:DD:EE:FF",
      pageId: 1,
      intervalSeconds: 60,
      durationType: "none",
    }),
  );
});

test("renders loop tasks and delegates task actions", async () => {
  const user = userEvent.setup();
  const task = {
    id: 1,
    pluginKind: "custom",
    pluginId: "7",
    name: "天气循环",
    deviceId: "AA:BB:CC:DD:EE:FF",
    pageId: 1,
    intervalSeconds: 60,
    durationType: "none" as const,
    status: "error" as const,
    errorMessage: "推送失败",
    createdAt: "2026-04-25T00:00:00Z",
    updatedAt: "2026-04-25T00:00:00Z",
  };
  const startLoopTask = vi.fn().mockResolvedValue({ ...task, status: "running" as const });
  const stopLoopTask = vi.fn().mockResolvedValue({ ...task, status: "idle" as const });
  const deleteLoopTask = vi.fn().mockResolvedValue(undefined);

  render(
    <PluginMarketPage
      devices={devices}
      builtinPlugins={[]}
      customPlugins={[]}
      pluginLoopTasks={[task]}
      onSavePlugin={vi.fn()}
      onDeletePlugin={vi.fn()}
      onPushPlugin={vi.fn()}
      onCreateLoopTask={vi.fn()}
      onDeleteLoopTask={deleteLoopTask}
      onStartLoopTask={startLoopTask}
      onStopLoopTask={stopLoopTask}
    />,
  );

  await user.click(screen.getByRole("tab", { name: "任务管理" }));

  expect(screen.getByText("天气循环")).toBeInTheDocument();
  expect(screen.getByText("第 1 页 · 每 60 秒 · error")).toBeInTheDocument();
  expect(screen.getByText("推送失败")).toBeInTheDocument();
  expect(screen.queryByText("暂无循环任务")).not.toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "启动" }));
  await user.click(screen.getByRole("button", { name: "停止" }));
  await user.click(screen.getByRole("button", { name: "删除" }));

  expect(startLoopTask).toHaveBeenCalledWith(1);
  expect(stopLoopTask).toHaveBeenCalledWith(1);
  expect(deleteLoopTask).toHaveBeenCalledWith(1);
});
