import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { AppSidebar } from "./app-sidebar";

test("reserves sidebar space for macOS window controls before the brand", () => {
  render(
    <MemoryRouter>
      <AppSidebar />
    </MemoryRouter>,
  );

  const controlsArea = screen.getByLabelText("macOS 窗口控制区");
  const brandText = screen.getByText("Zectrix Desktop");

  expect(controlsArea).toBeInTheDocument();
  expect(
    controlsArea.compareDocumentPosition(brandText) & Node.DOCUMENT_POSITION_FOLLOWING,
  ).toBeTruthy();
});
