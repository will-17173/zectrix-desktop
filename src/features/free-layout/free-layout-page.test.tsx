vi.mock("../../components/ui/toast", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { FreeLayoutPage } from "./free-layout-page";

test("pushes free layout text with selected font size and page", async () => {
  const user = userEvent.setup();
  const onPushText = vi.fn().mockResolvedValue(undefined);

  render(
    <FreeLayoutPage
      devices={[{ deviceId: "AA:BB", alias: "桌面屏", board: "note" }]}
      onPushText={onPushText}
    />,
  );

  await user.type(screen.getByLabelText("文本内容"), "行情内容");

  const fontSizeSelect = screen.getByRole("combobox", { name: "字号" });
  const pageSelect = screen.getByRole("combobox", { name: "页码" });

  expect(fontSizeSelect).toHaveAttribute("aria-labelledby", "font-size-label font-size-trigger");
  expect(pageSelect).toHaveAttribute("aria-labelledby", "page-id-label page-id-trigger");

  await user.click(fontSizeSelect);
  await user.click(await screen.findByRole("option", { name: "24px" }));
  expect(fontSizeSelect).toHaveTextContent("24px");

  await user.click(pageSelect);
  await user.click(await screen.findByRole("option", { name: "第 4 页" }));
  expect(pageSelect).toHaveTextContent("第 4 页");

  await user.click(screen.getByRole("button", { name: "推送" }));

  expect(onPushText).toHaveBeenCalledWith("行情内容", 24, "AA:BB", 4);
});