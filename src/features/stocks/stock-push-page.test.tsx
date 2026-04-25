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

const devices = [{ deviceId: "AA:BB", alias: "桌面屏", board: "note" }];
const watchlist = [{ code: "600519", createdAt: "2026-04-25T10:30:00Z" }];

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
      onAddStock={vi.fn()}
      onRemoveStock={vi.fn()}
      onPushStocks={vi.fn()}
    />,
  );

  expect(screen.getByRole("heading", { name: "股票推送" })).toBeInTheDocument();
  expect(screen.getByLabelText("股票代码")).toBeInTheDocument();
  expect(screen.getByText("600519")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "推送" })).toBeInTheDocument();
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
        onAddStock={onAddStock}
        onRemoveStock={vi.fn()}
        onPushStocks={vi.fn()}
      />
    </>,
  );

  await user.type(screen.getByLabelText("股票代码"), "830000");
  await user.click(screen.getByRole("button", { name: "添加" }));

  expect(onAddStock).not.toHaveBeenCalled();
  expect(await screen.findByText("仅支持 0、3、6 开头的 A 股代码")).toBeInTheDocument();
});

test("adds and removes stocks", async () => {
  const user = userEvent.setup();
  const onAddStock = vi.fn().mockResolvedValue({ code: "000001", createdAt: "2026-04-25T10:31:00Z" });
  const onRemoveStock = vi.fn().mockResolvedValue(undefined);

  render(
    <StockPushPage
      devices={devices}
      watchlist={watchlist}
      onAddStock={onAddStock}
      onRemoveStock={onRemoveStock}
      onPushStocks={vi.fn()}
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

  render(
    <StockPushPage
      devices={devices}
      watchlist={[
        { code: "600519", createdAt: "2026-04-25T10:30:00Z" },
        { code: "000001", createdAt: "2026-04-25T10:31:00Z" },
      ]}
      onAddStock={vi.fn()}
      onRemoveStock={onRemoveStock}
      onPushStocks={vi.fn()}
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
      onAddStock={vi.fn()}
      onRemoveStock={vi.fn()}
      onPushStocks={onPushStocks}
    />,
  );

  await user.click(screen.getByRole("combobox"));
  await user.click(await screen.findByRole("option", { name: "第 3 页" }));
  await user.click(screen.getByRole("button", { name: "推送" }));

  expect(onPushStocks).toHaveBeenCalledWith("AA:BB", 3);
});