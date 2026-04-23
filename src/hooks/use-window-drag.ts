import { useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function useWindowDrag() {
  const startDragging = useCallback((e: React.MouseEvent) => {
    // 只响应左键，且排除按钮/链接等交互元素
    if (e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (
      target.closest("button") ||
      target.closest("a") ||
      target.closest("input") ||
      target.closest("select") ||
      target.closest("[role='button']")
    ) {
      return;
    }
    void getCurrentWindow().startDragging();
  }, []);

  return startDragging;
}