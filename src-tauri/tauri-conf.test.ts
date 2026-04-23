import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, test } from "vitest";

describe("Tauri window chrome", () => {
  test("uses overlay titlebar and hides the native centered macOS title", () => {
    const configPath = join(process.cwd(), "src-tauri", "tauri.conf.json");
    const config = JSON.parse(readFileSync(configPath, "utf8")) as {
      app: { windows: Array<{ hiddenTitle?: boolean; titleBarStyle?: string }> };
    };

    expect(config.app.windows[0]?.titleBarStyle).toBe("Overlay");
    expect(config.app.windows[0]?.hiddenTitle).toBe(true);
  });
});
