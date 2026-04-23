import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/react";
import { ImageTemplatesPage } from "./image-templates-page";

test("opens the import dialog and exposes the local gallery actions", async () => {
  const user = userEvent.setup();
  const saveTemplate = vi.fn().mockResolvedValue({
    id: 1,
    name: "天气卡片",
    filePath: "/tmp/weather-card.png",
  });

  render(
    <ImageTemplatesPage
      templates={[{ id: 7, name: "现有图片", filePath: "/tmp/existing.png" }]}
      devices={[{ deviceId: "AA:BB:CC:DD:EE:FF", alias: "Desk", board: "bread-compact-wifi" }]}
      onSaveTemplate={saveTemplate}
      onPushTemplate={vi.fn()}
      onDeleteTemplate={vi.fn()}
    />,
  );

  expect(screen.getByRole("heading", { name: "本地图库" })).toBeInTheDocument();
  // 每张图片下方有页面选择器和推送按钮
  expect(screen.getByRole("combobox", { name: undefined })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "推送" })).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "导入图片" }));

  expect(await screen.findByLabelText("选择图片")).toBeInTheDocument();
  expect(screen.getByText("400x300 效果预览")).toBeInTheDocument();
});
