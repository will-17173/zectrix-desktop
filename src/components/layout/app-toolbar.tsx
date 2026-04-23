import type { SyncState } from "../../features/sync/sync-status";
import { useWindowDrag } from "../../hooks/use-window-drag";

type AppToolbarProps = {
  title: string;
  syncState: SyncState;
  syncMessage?: string | null;
  onSync?: () => void;
};

export function AppToolbar({ title, syncState, syncMessage, onSync }: AppToolbarProps) {
  const onDrag = useWindowDrag();
  return (
    <header className="app-toolbar" onMouseDown={onDrag}>
      <div className="app-toolbar-copy">
        <p className="app-toolbar-kicker">Workspace</p>
        <h1 className="app-toolbar-title">{title}</h1>
      </div>
      <div className="app-toolbar-actions">
        {syncState !== "idle" && syncState !== "syncing" && syncMessage ? (
          <span className="sync-feedback" data-state={syncState} aria-live="polite">
            {syncMessage}
          </span>
        ) : null}
        {onSync ? (
          <button className="app-toolbar-action" disabled={syncState === "syncing"} onClick={onSync} type="button">
            {syncState === "syncing" ? "同步中" : "同步"}
          </button>
        ) : null}
      </div>
    </header>
  );
}
