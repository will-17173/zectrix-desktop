import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/react";
import { TextTemplatesPage } from "./text-templates-page";

test("pushes text directly to a device with selected page and font size", async () => {
  const user = userEvent.setup();
  const pushText = vi.fn().mockResolvedValue(undefined);

  render(
    <TextTemplatesPage
      devices={[{ deviceId: "AA:BB:CC:DD:EE:FF", alias: "Desk", board: "bread-compact-wifi" }]}
      onPushText={pushText}
    />,
  );

  await user.type(screen.getByLabelText("文本内容"), "带钥匙\n带工牌");
  await user.selectOptions(screen.getByLabelText("字体大小"), "32");
  await user.selectOptions(screen.getByLabelText("目标页面"), "2");
  await user.click(screen.getByRole("button", { name: "推送" }));

  expect(pushText).toHaveBeenCalledWith("带钥匙\n带工牌", 32, "AA:BB:CC:DD:EE:FF", 2);
});