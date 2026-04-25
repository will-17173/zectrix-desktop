const toastHarness = vi.hoisted(() => {
  const listeners = new Set<() => void>();
  let message = "";

  const notify = () => {
    listeners.forEach((listener) => listener());
  };

  const setMessage = (nextMessage: string) => {
    message = nextMessage;
    notify();
  };

  const toast = {
    error: vi.fn((nextMessage: string) => {
      setMessage(nextMessage);
    }),
    success: vi.fn((nextMessage?: string) => {
      if (nextMessage) {
        setMessage(nextMessage);
      }
    }),
  };

  return {
    toast,
    subscribe(listener: () => void) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    getSnapshot() {
      return message;
    },
    reset() {
      message = "";
      toast.error.mockClear();
      toast.success.mockClear();
      notify();
    },
  };
});

vi.mock("../../components/ui/toast", async () => {
  const React = await import("react");

  return {
    Toaster: () => {
      const message = React.useSyncExternalStore(
        toastHarness.subscribe,
        toastHarness.getSnapshot,
      );

      return message
        ? React.createElement("div", { role: "alert" }, message)
        : null;
    },
    toast: toastHarness.toast,
  };
});

import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Toaster } from "../../components/ui/toast";
import { StockPushPage } from "./stock-push-page";
import type { StockPushTaskRecord } from "../../lib/tauri";

const devices = [{ deviceId: "AA:BB", alias: "桌面屏", board: "note" }];
const watchlist = [{ code: "600519", createdAt: "2026-04-25T10:30:00Z" }];
const defaultQuotes = [
  { code: "600519", name: "贵州茅台", price: 1458.49, change: 39.49, changePercent: 2.78, valid: true },
];

const defaultPushTask: StockPushTaskRecord = {
  id: 1,
  deviceId: "AA:BB",
  pageId: 1,
  intervalSeconds: 60,
  status: "stopped",
  errorMessage: undefined,
  startedAt: undefined,
  lastPushAt: undefined,
  createdAt: "2026-04-25T10:00:00Z",
  updatedAt: "2026-04-25T10:00:00Z",
};

function createDeferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((nextResolve, nextReject) => {
    resolve = nextResolve;
    reject = nextReject;
  });

  return { promise, resolve, reject };
}

beforeEach(() => {
  toastHarness.reset();
});

test("renders stock controls and existing watchlist", () => {
  render(
    <StockPushPage
      devices={devices}
      watchlist={watchlist}
      quotes={defaultQuotes}
      pushTask={null}
      onAddStock={vi.fn()}
      onRemoveStock={vi.fn()}
      onPushStocks={vi.fn()}
      onFetchQuotes={vi.fn().mockResolvedValue(defaultQuotes)}
      onCreateTask={vi.fn()}
      onStartTask={vi.fn()}
      onStopTask={vi.fn()}
    />,
  );

  expect(screen.getByRole("heading", { name: "股票推送" })).toBeInTheDocument();
  expect(screen.getByLabelText("股票代码")).toBeInTheDocument();
  expect(screen.getByText("600519")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "单次推送" })).toBeInTheDocument();
});

test("rejects invalid stock code before calling add", async () => {
  const user = userEvent.setup();
  const onAddStock = vi.fn();

  render(
    <>
      <Toaster position="top-center" />
      <StockPushPage
        devices={devices}
        watchlist={[]}
        quotes={[]}
        pushTask={null}
        onAddStock={onAddStock}
        onRemoveStock={vi.fn()}
        onPushStocks={vi.fn()}
        onFetchQuotes={vi.fn().mockResolvedValue([])}
        onCreateTask={vi.fn()}
        onStartTask={vi.fn()}
        onStopTask={vi.fn()}
      />
    </>,
  );

  await user.type(screen.getByLabelText("股票代码"), "abc");
  await user.click(screen.getByRole("button", { name: "添加" }));

  expect(onAddStock).not.toHaveBeenCalled();
  expect(await screen.findByText("股票代码必须是 6 位数字")).toBeInTheDocument();
});

test("adds and removes stocks", async () => {
  const user = userEvent.setup();
  const onAddStock = vi.fn().mockResolvedValue({ code: "000001", createdAt: "2026-04-25T10:31:00Z" });
  const onRemoveStock = vi.fn().mockResolvedValue(undefined);

  render(
    <StockPushPage
      devices={devices}
      watchlist={watchlist}
      quotes={defaultQuotes}
      pushTask={null}
      onAddStock={onAddStock}
      onRemoveStock={onRemoveStock}
      onPushStocks={vi.fn()}
      onFetchQuotes={vi.fn().mockResolvedValue(defaultQuotes)}
      onCreateTask={vi.fn()}
      onStartTask={vi.fn()}
      onStopTask={vi.fn()}
    />,
  );

  await user.type(screen.getByLabelText("股票代码"), "000001");
  await user.click(screen.getByRole("button", { name: "添加" }));
  await user.click(screen.getByRole("button", { name: "删除 600519" }));

  expect(onAddStock).toHaveBeenCalledWith("000001");
  expect(onRemoveStock).toHaveBeenCalledWith("600519");
});

test("keeps both clicked delete buttons disabled during concurrent removals until each request finishes", async () => {
  const user = userEvent.setup();
  const firstRemoval = createDeferred<void>();
  const secondRemoval = createDeferred<void>();
  const onRemoveStock = vi.fn((code: string) => {
    if (code === "600519") {
      return firstRemoval.promise;
    }

    if (code === "000001") {
      return secondRemoval.promise;
    }

    return Promise.reject(new Error(`unexpected stock code: ${code}`));
  });

  const twoQuotes = [
    { code: "600519", name: "贵州茅台", price: 1458.49, change: 39.49, changePercent: 2.78, valid: true },
    { code: "000001", name: "平安银行", price: 11.0, change: 0.0, changePercent: 0.0, valid: true },
  ];

  render(
    <StockPushPage
      devices={devices}
      watchlist={[
        { code: "600519", createdAt: "2026-04-25T10:30:00Z" },
        { code: "000001", createdAt: "2026-04-25T10:31:00Z" },
      ]}
      quotes={twoQuotes}
      pushTask={null}
      onAddStock={vi.fn()}
      onRemoveStock={onRemoveStock}
      onPushStocks={vi.fn()}
      onFetchQuotes={vi.fn().mockResolvedValue(twoQuotes)}
      onCreateTask={vi.fn()}
      onStartTask={vi.fn()}
      onStopTask={vi.fn()}
    />,
  );

  const firstButton = screen.getByRole("button", { name: "删除 600519" });
  const secondButton = screen.getByRole("button", { name: "删除 000001" });

  await user.click(firstButton);
  await user.click(secondButton);

  expect(firstButton).toBeDisabled();
  expect(secondButton).toBeDisabled();

  firstRemoval.resolve();
  await waitFor(() => {
    expect(screen.queryByText("600519")).not.toBeInTheDocument();
  });

  expect(screen.getByRole("button", { name: "删除 000001" })).toBeDisabled();

  secondRemoval.resolve();
  await waitFor(() => {
    expect(screen.queryByText("000001")).not.toBeInTheDocument();
  });

  expect(onRemoveStock).toHaveBeenCalledTimes(2);
});

test("pushes stocks to first device and selected page", async () => {
  const user = userEvent.setup();
  const onPushStocks = vi.fn().mockResolvedValue(undefined);

  render(
    <StockPushPage
      devices={devices}
      watchlist={watchlist}
      quotes={defaultQuotes}
      pushTask={null}
      onAddStock={vi.fn()}
      onRemoveStock={vi.fn()}
      onPushStocks={onPushStocks}
      onFetchQuotes={vi.fn().mockResolvedValue(defaultQuotes)}
      onCreateTask={vi.fn()}
      onStartTask={vi.fn()}
      onStopTask={vi.fn()}
    />,
  );

  await user.click(screen.getByRole("combobox", { name: "目标页面" }));
  await user.click(await screen.findByRole("option", { name: "第 3 页" }));
  await user.click(screen.getByRole("button", { name: "单次推送" }));

  expect(onPushStocks).toHaveBeenCalledWith("AA:BB", 3);
});

test("shows start loop button when task is not running", () => {
  render(
    <StockPushPage
      devices={devices}
      watchlist={watchlist}
      quotes={defaultQuotes}
      pushTask={defaultPushTask}
      onAddStock={vi.fn()}
      onRemoveStock={vi.fn()}
      onPushStocks={vi.fn()}
      onFetchQuotes={vi.fn().mockResolvedValue(defaultQuotes)}
      onCreateTask={vi.fn()}
      onStartTask={vi.fn()}
      onStopTask={vi.fn()}
    />,
  );

  expect(screen.getByRole("button", { name: "开始循环" })).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "停止循环" })).not.toBeInTheDocument();
});

test("shows stop loop button when task is running", () => {
  const runningTask: StockPushTaskRecord = {
    ...defaultPushTask,
    status: "running",
    startedAt: "2026-04-25T11:00:00Z",
    lastPushAt: "2026-04-25T11:00:00Z",
  };

  render(
    <StockPushPage
      devices={devices}
      watchlist={watchlist}
      quotes={defaultQuotes}
      pushTask={runningTask}
      onAddStock={vi.fn()}
      onRemoveStock={vi.fn()}
      onPushStocks={vi.fn()}
      onFetchQuotes={vi.fn().mockResolvedValue(defaultQuotes)}
      onCreateTask={vi.fn()}
      onStartTask={vi.fn()}
      onStopTask={vi.fn()}
    />,
  );

  expect(screen.getByRole("button", { name: "停止循环" })).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "开始循环" })).not.toBeInTheDocument();
  expect(screen.getByText("运行中")).toBeInTheDocument();
});

test("starts loop task with correct parameters", async () => {
  const user = userEvent.setup();
  const onCreateTask = vi.fn().mockResolvedValue(defaultPushTask);
  const onStartTask = vi.fn().mockResolvedValue({ ...defaultPushTask, status: "running" });

  render(
    <>
      <Toaster position="top-center" />
      <StockPushPage
        devices={devices}
        watchlist={watchlist}
        quotes={defaultQuotes}
        pushTask={null}
        onAddStock={vi.fn()}
        onRemoveStock={vi.fn()}
        onPushStocks={vi.fn()}
        onFetchQuotes={vi.fn().mockResolvedValue(defaultQuotes)}
        onCreateTask={onCreateTask}
        onStartTask={onStartTask}
        onStopTask={vi.fn()}
      />
    </>,
  );

  // Select interval
  await user.click(screen.getByRole("combobox", { name: "推送间隔" }));
  await user.click(await screen.findByRole("option", { name: "5 分钟" }));

  // Select page
  await user.click(screen.getByRole("combobox", { name: "目标页面" }));
  await user.click(await screen.findByRole("option", { name: "第 3 页" }));

  // Start loop
  await user.click(screen.getByRole("button", { name: "开始循环" }));

  expect(onCreateTask).toHaveBeenCalledWith("AA:BB", 3, 300);
  expect(onStartTask).toHaveBeenCalled();
  expect(await screen.findByText("循环推送已启动")).toBeInTheDocument();
});

test("stops loop task", async () => {
  const user = userEvent.setup();
  const onStopTask = vi.fn().mockResolvedValue(defaultPushTask);

  const runningTask: StockPushTaskRecord = {
    ...defaultPushTask,
    status: "running",
  };

  render(
    <>
      <Toaster position="top-center" />
      <StockPushPage
        devices={devices}
        watchlist={watchlist}
        quotes={defaultQuotes}
        pushTask={runningTask}
        onAddStock={vi.fn()}
        onRemoveStock={vi.fn()}
        onPushStocks={vi.fn()}
        onFetchQuotes={vi.fn().mockResolvedValue(defaultQuotes)}
        onCreateTask={vi.fn()}
        onStartTask={vi.fn()}
        onStopTask={onStopTask}
      />
    </>,
  );

  await user.click(screen.getByRole("button", { name: "停止循环" }));

  expect(onStopTask).toHaveBeenCalled();
  expect(await screen.findByText("循环推送已停止")).toBeInTheDocument();
});