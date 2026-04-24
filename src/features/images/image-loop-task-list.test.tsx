import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ImageLoopTaskList } from "./image-loop-task-list";
import type { ImageLoopTask, DeviceRecord } from "../../lib/tauri";

const mockDevice: DeviceRecord = {
  deviceId: "AA:BB:CC:DD:EE:FF",
  alias: "测试设备",
  board: "test",
  cachedAt: "2026-04-24T10:00:00Z",
  apiKeyId: 1,
};

const mockTask: ImageLoopTask = {
  id: 1,
  name: "周末旅行",
  folderPath: "/Users/test/Pictures/travel",
  deviceId: "AA:BB:CC:DD:EE:FF",
  pageId: 1,
  intervalSeconds: 30,
  durationType: "none",
  status: "idle",
  currentIndex: 0,
  totalImages: 12,
  createdAt: "2026-04-24T10:00:00Z",
  updatedAt: "2026-04-24T10:00:00Z",
};

describe("ImageLoopTaskList", () => {
  it("shows empty message when no tasks", () => {
    render(
      <ImageLoopTaskList
        tasks={[]}
        devices={[mockDevice]}
        onStart={vi.fn()}
        onStop={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    expect(screen.getByText(/暂无循环相册任务/)).toBeInTheDocument();
  });

  it("renders task cards for each task", () => {
    render(
      <ImageLoopTaskList
        tasks={[mockTask]}
        devices={[mockDevice]}
        onStart={vi.fn()}
        onStop={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    expect(screen.getByText("任务名称: 周末旅行")).toBeInTheDocument();
    expect(screen.getByText("未启动")).toBeInTheDocument();
  });

  it("shows running status for running task", () => {
    const runningTask = { ...mockTask, status: "running" as const };
    render(
      <ImageLoopTaskList
        tasks={[runningTask]}
        devices={[mockDevice]}
        onStart={vi.fn()}
        onStop={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    expect(screen.getByText(/当前: 第/)).toBeInTheDocument();
  });
});
