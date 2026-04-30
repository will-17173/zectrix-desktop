import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CalendarSyncPanel } from "./calendar-sync-panel";

const {
  getCalendarSyncConfig,
  saveCalendarSyncConfig,
  listCalendars,
  syncCalendar,
} = vi.hoisted(() => ({
  getCalendarSyncConfig: vi.fn().mockResolvedValue({
    enabled: false,
    direction: "ToCalendar",
    targetType: "Reminder",
    targetCalendarId: null,
  }),
  saveCalendarSyncConfig: vi.fn().mockResolvedValue(undefined),
  listCalendars: vi.fn().mockResolvedValue([
    { id: "cal-1", title: "提醒事项", color: "#ff0000" },
  ]),
  syncCalendar: vi.fn().mockResolvedValue({ created: 2, updated: 1, skipped: 0, deleted: 0 }),
}));

vi.mock("../../lib/tauri", () => ({
  getCalendarSyncConfig,
  saveCalendarSyncConfig,
  listCalendars,
  syncCalendar,
}));

test("renders calendar sync section heading", async () => {
  render(<CalendarSyncPanel />);
  expect(await screen.findByText("启用日历同步")).toBeInTheDocument();
});

test("enable toggle calls saveCalendarSyncConfig", async () => {
  const user = userEvent.setup();
  render(<CalendarSyncPanel />);
  const toggle = await screen.findByRole("checkbox", { name: /启用日历同步/ });
  await user.click(toggle);
  expect(saveCalendarSyncConfig).toHaveBeenCalledWith(
    expect.objectContaining({ enabled: true })
  );
});

test("shows calendar options when enabled", async () => {
  getCalendarSyncConfig.mockResolvedValueOnce({
    enabled: true,
    direction: "ToCalendar",
    targetType: "Reminder",
    targetCalendarId: null,
  });
  render(<CalendarSyncPanel />);
  await screen.findByText("目标类型");
  expect(screen.getByText("同步方向")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "立即同步" })).toBeInTheDocument();
});

test("sync button shows result summary", async () => {
  getCalendarSyncConfig.mockResolvedValueOnce({
    enabled: true,
    direction: "ToCalendar",
    targetType: "Reminder",
    targetCalendarId: "cal-1",
  });
  const user = userEvent.setup();
  render(<CalendarSyncPanel />);
  const syncBtn = await screen.findByRole("button", { name: "立即同步" });
  await user.click(syncBtn);
  await waitFor(() => {
    expect(screen.getByText(/新增 2 条/)).toBeInTheDocument();
  });
});
