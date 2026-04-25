import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/react";
import { TextTemplatesPage } from "./text-templates-page";

test("pushes structured text directly to a device with selected page", async () => {
  const user = userEvent.setup();
  const pushText = vi.fn().mockResolvedValue(undefined);

  render(
    <TextTemplatesPage
      devices={[{ deviceId: "AA:BB:CC:DD:EE:FF", alias: "Desk", board: "bread-compact-wifi" }]}
      onPushText={pushText}
    />,
  );

  await user.type(screen.getByLabelText("标题"), "出门提醒");
  await user.type(screen.getByLabelText("正文"), "带钥匙\n带工牌");
  await user.click(screen.getByRole("combobox", { name: "目标页面" }));
  await user.click(screen.getByRole("option", { name: "第 2 页" }));
  expect(screen.getByRole("combobox", { name: "目标页面" })).toHaveTextContent("第 2 页");
  await user.click(screen.getByRole("button", { name: "推送" }));

  expect(screen.queryByLabelText("字体大小")).not.toBeInTheDocument();
  expect(pushText).toHaveBeenCalledWith("出门提醒", "带钥匙\n带工牌", "AA:BB:CC:DD:EE:FF", 2);
});
