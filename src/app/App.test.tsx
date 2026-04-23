import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { MemoryRouter } from "react-router-dom";
import { loadBootstrapState } from "../lib/tauri";
import App from "./App";

const mockSyncAll = vi.fn();

vi.mock("../lib/tauri", () => ({
  loadBootstrapState: vi.fn().mockResolvedValue({
    apiKeys: [],
    devices: [],
    todos: [],
    textTemplates: [],
    imageTemplates: [],
    lastSyncTime: null,
  }),
  createLocalTodo: vi.fn(),
  toggleTodoStatus: vi.fn(),
  deleteLocalTodo: vi.fn(),
  updateLocalTodo: vi.fn(),
  pushTodoToDevice: vi.fn(),
  pushText: vi.fn(),
  saveImageTemplate: vi.fn(),
  pushImageTemplate: vi.fn(),
  addApiKey: vi.fn(),
  removeApiKey: vi.fn(),
  addDeviceCache: vi.fn(),
  removeDeviceCache: vi.fn(),
  syncAll: (...args: unknown[]) => mockSyncAll(...args),
}));

beforeEach(() => {
  mockSyncAll.mockReset();
  vi.mocked(loadBootstrapState).mockResolvedValue({
    apiKeys: [],
    devices: [],
    todos: [],
    textTemplates: [],
    imageTemplates: [],
    lastSyncTime: null,
  });
});

test("renders the four navigation entries and onboarding message", async () => {
  render(
    <MemoryRouter>
      <App />
    </MemoryRouter>,
  );

  expect(await screen.findByRole("navigation")).toBeInTheDocument();
  expect(screen.getAllByText("待办事项").length).toBeGreaterThanOrEqual(1);
  expect(screen.getByText("图片推送")).toBeInTheDocument();
  expect(screen.getByText("文本推送")).toBeInTheDocument();
  expect(screen.getByText("设置")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "添加待办" })).toBeInTheDocument();
});

test("hides sync button when no API key is configured", async () => {
  render(
    <MemoryRouter>
      <App />
    </MemoryRouter>,
  );

  await screen.findByRole("navigation");
  expect(screen.queryByRole("button", { name: "同步" })).not.toBeInTheDocument();
});

test("runs manual sync and shows success feedback", async () => {
  const { loadBootstrapState } = await import("../lib/tauri");
  vi.mocked(loadBootstrapState).mockResolvedValue({
    apiKeys: [{ id: 1, name: "test", key: "zt_test", createdAt: "2026-04-23T00:00:00Z" }],
    devices: [],
    todos: [],
    textTemplates: [],
    imageTemplates: [],
    lastSyncTime: null,
  });

  mockSyncAll.mockResolvedValue({
    apiKeys: [{ id: 1, name: "test", key: "zt_test", createdAt: "2026-04-23T00:00:00Z" }],
    devices: [{ deviceId: "AA:BB", alias: "synced", board: "b", apiKeyId: 1 }],
    todos: [],
    textTemplates: [],
    imageTemplates: [],
    lastSyncTime: "2026-04-23T00:00:00Z",
  });

  render(
    <MemoryRouter>
      <App />
    </MemoryRouter>,
  );

  const syncBtn = await screen.findByRole("button", { name: "同步" });
  await userEvent.click(syncBtn);

  expect(mockSyncAll).toHaveBeenCalled();

  expect(screen.getByText("同步成功")).toBeInTheDocument();
});

test("shows sync failure feedback when manual sync throws", async () => {
  const { loadBootstrapState } = await import("../lib/tauri");
  vi.mocked(loadBootstrapState).mockResolvedValue({
    apiKeys: [{ id: 1, name: "test", key: "zt_test", createdAt: "2026-04-23T00:00:00Z" }],
    devices: [],
    todos: [],
    textTemplates: [],
    imageTemplates: [],
    lastSyncTime: null,
  });

  mockSyncAll.mockRejectedValueOnce(new Error("boom"));

  render(
    <MemoryRouter>
      <App />
    </MemoryRouter>,
  );

  await userEvent.click(await screen.findByRole("button", { name: "同步" }));

  expect(await screen.findByText("同步失败: boom")).toBeInTheDocument();
  expect(screen.getByText("同步失败: boom")).toBeInTheDocument();
});

test("renders the redesigned shell chrome and marks the active navigation item", async () => {
  render(
    <MemoryRouter initialEntries={["/text-push"]}>
      <App />
    </MemoryRouter>,
  );

  expect(await screen.findByRole("navigation", { name: "主导航" })).toBeInTheDocument();
  expect(screen.getByRole("main")).toHaveClass("app-main");
  expect(screen.getByLabelText("应用工作区外框")).toHaveClass("app-main-frame");
  expect(screen.getByLabelText("主内容画布")).toHaveClass("app-canvas");

  const activeLink = screen.getByRole("link", { name: "文本推送" });
  expect(activeLink).toHaveAttribute("aria-current", "page");
});

test("renders settings in the sidebar footer group", async () => {
  render(
    <MemoryRouter>
      <App />
    </MemoryRouter>,
  );

  await screen.findByRole("navigation", { name: "主导航" });

  const primaryNav = screen.getByRole("list", { name: "主功能" });
  const footerNav = screen.getByRole("list", { name: "底部功能" });

  expect(within(primaryNav).queryByRole("link", { name: "设置" })).not.toBeInTheDocument();
  expect(within(footerNav).getByRole("link", { name: "设置" })).toBeInTheDocument();
});

test("renders the redesigned toolbar title, sync action, and compact status badge", async () => {
  const { loadBootstrapState } = await import("../lib/tauri");
  vi.mocked(loadBootstrapState).mockResolvedValue({
    apiKeys: [{ id: 1, name: "test", key: "zt_test", createdAt: "2026-04-23T00:00:00Z" }],
    devices: [],
    todos: [],
    textTemplates: [],
    imageTemplates: [],
    lastSyncTime: null,
  });

  render(
    <MemoryRouter>
      <App />
    </MemoryRouter>,
  );

  expect(await screen.findByRole("banner")).toHaveClass("app-toolbar");
  expect(screen.getByRole("heading", { name: "待办事项" })).toHaveClass("app-toolbar-title");
  expect(screen.getByRole("button", { name: "同步" })).toHaveClass("app-toolbar-action");
});

test("applies the new shell utility classes required by the Playground-lite layout", async () => {
  render(
    <MemoryRouter>
      <App />
    </MemoryRouter>,
  );

  await screen.findByRole("navigation", { name: "主导航" });

  expect(document.documentElement).toHaveClass("theme-shell");
  expect(screen.getByRole("navigation", { name: "主导航" })).toHaveClass("app-sidebar");
  expect(screen.getByLabelText("主内容画布")).toHaveClass("app-canvas");
});

test("declares the bundled font files and keeps the requested global font priority", () => {
  const cssPath = resolve(process.cwd(), "src/App.css");
  const css = readFileSync(cssPath, "utf8");

  expect(css).toContain('url("./assets/fonts/ShareTechMono-Regular.ttf")');
  expect(css).toContain('url("./assets/fonts/MonuTitl-0.95CnMd.woff2")');
  expect(css).toMatch(/font-family:\s*"ShareTechMono",\s*"MonuTitl"/);
});
