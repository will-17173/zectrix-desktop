import { NavLink } from "react-router-dom";
import { CheckSquare, FileText, Image, Settings, PenTool, LayoutGrid } from "lucide-react";
import { useWindowDrag } from "../../hooks/use-window-drag";

const primaryNavItems = [
  { label: "待办事项", icon: CheckSquare, href: "/" },
  { label: "涂鸦推送", icon: PenTool, href: "/sketch-push" },
  { label: "图片推送", icon: Image, href: "/image-push" },
  { label: "自由排版", icon: LayoutGrid, href: "/free-layout" },
  { label: "文本推送", icon: FileText, href: "/text-push" },
];

const secondaryNavItem = { label: "设置", icon: Settings, href: "/settings" };

function SidebarLink({
  href,
  label,
  icon: Icon,
}: {
  href: string;
  label: string;
  icon: typeof CheckSquare;
}) {
  return (
    <NavLink
      to={href}
      end={href === "/"}
      className={({ isActive }) =>
        isActive ? "app-nav-link is-active" : "app-nav-link"
      }
    >
      <Icon size={18} strokeWidth={1.75} />
      <span>{label}</span>
    </NavLink>
  );
}

export function AppSidebar() {
  const onDrag = useWindowDrag();
  return (
    <nav className="app-sidebar" aria-label="主导航" onMouseDown={onDrag}>
      <div className="app-sidebar-inner">
        <div className="app-sidebar-window-controls" aria-label="macOS 窗口控制区" />
        <div className="app-sidebar-brand">
          <span className="app-sidebar-brand-mark" aria-hidden="true" />
          <span className="app-sidebar-brand-text">Zectrix Desktop</span>
        </div>
        <ul className="app-sidebar-list" aria-label="主功能">
          {primaryNavItems.map((item) => (
            <li key={item.href}>
              <SidebarLink {...item} />
            </li>
          ))}
        </ul>
        <div className="app-sidebar-footer">
          <ul className="app-sidebar-list" aria-label="底部功能">
            <li>
              <SidebarLink {...secondaryNavItem} />
            </li>
          </ul>
        </div>
      </div>
    </nav>
  );
}
