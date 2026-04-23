import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/react";
import { DeviceManagementPage } from "./device-management-page";

const addDevice = vi.fn().mockResolvedValue({
  deviceId: "AA:BB:CC:DD:EE:FF",
  alias: "Desk",
  board: "bread-compact-wifi",
});

test("validates and adds a locally cached device", async () => {
  const user = userEvent.setup();

  render(
    <DeviceManagementPage
      devices={[]}
      onAddDevice={addDevice}
      onRemoveDevice={vi.fn()}
    />,
  );

  await user.type(screen.getByLabelText("MAC 地址"), "AA:BB:CC:DD:EE:FF");
  await user.click(screen.getByRole("button", { name: "添加设备" }));

  expect(addDevice).toHaveBeenCalledWith("AA:BB:CC:DD:EE:FF");
  expect(await screen.findByText("Desk")).toBeInTheDocument();
});
