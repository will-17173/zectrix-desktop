import { useWindowDrag } from "../../hooks/use-window-drag";

type AppToolbarProps = {
  title: string;
};

export function AppToolbar({ title }: AppToolbarProps) {
  const onDrag = useWindowDrag();
  return (
    <header className="app-toolbar" onMouseDown={onDrag}>
      <div className="app-toolbar-copy">
        <p className="app-toolbar-kicker">Workspace</p>
        <h1 className="app-toolbar-title">{title}</h1>
      </div>
      <div className="app-toolbar-actions" />
    </header>
  );
}