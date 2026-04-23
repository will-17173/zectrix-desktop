import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/react";
import { TextTemplatesPage } from "./text-templates-page";

test("creates a structured text template and pushes it to a device", async () => {
  const user = userEvent.setup();
  const createTemplate = vi.fn().mockResolvedValue({
    id: 1,
    title: "晨间提醒",
    content: "带钥匙\n带工牌",
  });
  const pushTemplate = vi.fn().mockResolvedValue(undefined);

  render(
    <TextTemplatesPage
      templates={[]}
      devices={[{ deviceId: "AA:BB:CC:DD:EE:FF", alias: "Desk", board: "bread-compact-wifi" }]}
      onCreateTemplate={createTemplate}
      onPushTemplate={pushTemplate}
    />,
  );

  await user.type(screen.getByLabelText("模板标题"), "晨间提醒");
  await user.type(screen.getByLabelText("模板正文"), "带钥匙\n带工牌");
  await user.click(screen.getByRole("button", { name: "保存模板" }));
  await user.click(await screen.findByRole("button", { name: "推送模板" }));

  expect(createTemplate).toHaveBeenCalled();
  expect(pushTemplate).toHaveBeenCalledWith(1, "AA:BB:CC:DD:EE:FF");
});
