import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./select";

function TestSelect() {
  const [value, setValue] = useState("");

  return (
    <div>
      <label id="device-select-label" htmlFor="device-select-trigger">设备</label>
      <Select value={value} onValueChange={setValue}>
        <SelectTrigger id="device-select-trigger" aria-labelledby="device-select-label device-select-trigger" className="w-56">
          <SelectValue placeholder="不指定" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="desk">Desk</SelectItem>
          <SelectItem value="lab">Lab</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}

test("renders placeholder and updates selected value through the shadcn select", async () => {
  const user = userEvent.setup();

  render(<TestSelect />);

  const trigger = screen.getByRole("combobox", { name: "设备" });
  expect(trigger).toHaveTextContent("不指定");

  await user.click(trigger);
  await user.click(screen.getByRole("option", { name: "Desk" }));

  expect(trigger).toHaveTextContent("Desk");
});
