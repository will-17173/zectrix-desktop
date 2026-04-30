import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/react";
import { SettingsPage } from "./settings-page";

const { addApiKey, removeApiKey, addDeviceCache, removeDeviceCache, getCurrentVersion, checkForUpdate, getCalendarSyncConfig, saveCalendarSyncConfig, listCalendars, syncCalendar } = vi.hoisted(() => ({
  addApiKey: vi.fn().mockResolvedValue({ id: 1, name: "test", key: "zt_test", createdAt: "2026-04-23T00:00:00Z" }),
  removeApiKey: vi.fn().mockResolvedValue(undefined),
  addDeviceCache: vi.fn().mockResolvedValue({ deviceId: "AA:BB:CC:DD:EE:FF", alias: "Test Device", board: "board", apiKeyId: 1 }),
  removeDeviceCache: vi.fn().mockResolvedValue(undefined),
  getCurrentVersion: vi.fn(() => new Promise<string>(() => {})),
  checkForUpdate: vi.fn().mockResolvedValue(null),
  getCalendarSyncConfig: vi.fn().mockResolvedValue({ enabled: false, direction: "ToCalendar", targetType: "Reminder", targetCalendarId: null }),
  saveCalendarSyncConfig: vi.fn().mockResolvedValue(undefined),
  listCalendars: vi.fn().mockResolvedValue([]),
  syncCalendar: vi.fn().mockResolvedValue({ created: 0, updated: 0, skipped: 0, deleted: 0 }),
}));

vi.mock("../../lib/tauri", () => ({
  addApiKey,
  removeApiKey,
  addDeviceCache,
  removeDeviceCache,
  getCurrentVersion,
  checkForUpdate,
  getCalendarSyncConfig,
  saveCalendarSyncConfig,
  listCalendars,
  syncCalendar,
}));

test("adds and removes API keys", async () => {
  const user = userEvent.setup();

  render(
    <SettingsPage
      apiKeys={[]}
      devices={[]}
      onAddApiKey={addApiKey}
      onRemoveApiKey={removeApiKey}
      onAddDevice={addDeviceCache}
      onRemoveDevice={removeDeviceCache}
    />
  );

  await user.type(screen.getByLabelText("名称"), "test");
  await user.type(screen.getByLabelText("API Key"), "zt_test_key");
  await user.click(screen.getByRole("button", { name: "保存 API Key" }));
  expect(addApiKey).toHaveBeenCalledWith("test", "zt_test_key");
});

test("shows API key creation hint", async () => {
  render(
    <SettingsPage
      apiKeys={[]}
      devices={[]}
      onAddApiKey={addApiKey}
      onRemoveApiKey={removeApiKey}
      onAddDevice={addDeviceCache}
      onRemoveDevice={removeDeviceCache}
    />
  );

  expect(screen.getByText(/cloud.zectrix.com\/home\/api-keys/)).toBeInTheDocument();
});

test("collapses API key form behind an add button when keys already exist", async () => {
  const user = userEvent.setup();

  render(
    <SettingsPage
      apiKeys={[{ id: 1, name: "已有 Key", key: "zt_existing", createdAt: "2026-04-23T00:00:00Z" }]}
      devices={[]}
      onAddApiKey={addApiKey}
      onRemoveApiKey={removeApiKey}
      onAddDevice={addDeviceCache}
      onRemoveDevice={removeDeviceCache}
    />
  );

  expect(screen.queryByLabelText("名称")).not.toBeInTheDocument();
  expect(screen.getByRole("button", { name: "添加 API Key" })).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "添加 API Key" }));

  expect(screen.getByLabelText("名称")).toBeInTheDocument();
  expect(screen.getByLabelText("API Key")).toBeInTheDocument();
});

test("adds a device with the selected api key from the shared ui select", async () => {
  const user = userEvent.setup();

  render(
    <SettingsPage
      apiKeys={[{ id: 1, name: "已有 Key", key: "zt_existing", createdAt: "2026-04-23T00:00:00Z" }]}
      devices={[]}
      onAddApiKey={addApiKey}
      onRemoveApiKey={removeApiKey}
      onAddDevice={addDeviceCache}
      onRemoveDevice={removeDeviceCache}
    />
  );

  await user.click(screen.getByRole("combobox", { name: "选择 API Key" }));
  await user.click(screen.getByRole("option", { name: "已有 Key" }));
  await user.type(screen.getByLabelText("MAC 地址"), "aa:bb:cc:dd:ee:ff");
  await user.click(screen.getByRole("button", { name: "保存设备" }));

  expect(addDeviceCache).toHaveBeenCalledWith("AA:BB:CC:DD:EE:FF", 1);
});

test("shows error when removing API key with associated devices", async () => {
  const user = userEvent.setup();
  const removeApiKeyWithError = vi.fn().mockRejectedValue(new Error("该 API Key 有关联设备，请先删除关联设备"));

  render(
    <SettingsPage
      apiKeys={[{ id: 1, name: "已有 Key", key: "zt_existing", createdAt: "2026-04-23T00:00:00Z" }]}
      devices={[{ deviceId: "AA:BB:CC:DD:EE:FF", alias: "Test Device", board: "board", cachedAt: "2026-04-23T00:00:00Z", apiKeyId: 1 }]}
      onAddApiKey={addApiKey}
      onRemoveApiKey={removeApiKeyWithError}
      onAddDevice={addDeviceCache}
      onRemoveDevice={removeDeviceCache}
    />
  );

  // Click the delete button in the API Key section (first delete button)
  const deleteButtons = screen.getAllByRole("button", { name: "删除" });
  await user.click(deleteButtons[0]);

  expect(removeApiKeyWithError).toHaveBeenCalledWith(1);
  expect(screen.getByRole("alert")).toHaveTextContent("该 API Key 有关联设备，请先删除关联设备");
});
