import { NavLink } from "react-router-dom";
import { CheckSquare, FileText, Image, Settings, PenTool, LayoutGrid, Layers } from "lucide-react";
import { useWindowDrag } from "../../hooks/use-window-drag";

export function GithubIcon({ size = 16 }: { size?: number }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="currentColor"
    >
      <path d="M12 2A10 10 0 0 0 2 12c0 4.42 2.87 8.17 6.84 9.5c.5.08.66-.23.66-.5v-1.69c-2.77.6-3.36-1.34-3.36-1.34c-.46-1.16-1.11-1.47-1.11-1.47c-.91-.62.07-.6.07-.6c1 .07 1.53 1.03 1.53 1.03c.87 1.52 2.34 1.07 2.91.83c.09-.65.35-1.09.63-1.34c-2.22-.25-4.55-1.11-4.55-4.92c0-1.11.38-2 1.03-2.71c-.1-.25-.45-1.29.1-2.64c0 0 .84-.27 2.75 1.02c.79-.22 1.65-.33 2.5-.33s1.71.11 2.5.33c1.91-1.29 2.75-1.02 2.75-1.02c.55 1.35.2 2.39.1 2.64c.65.71 1.03 1.6 1.03 2.71c0 3.82-2.34 4.66-4.57 4.91c.36.31.69.92.69 1.85V21c0 .27.16.59.67.5C19.14 20.16 22 16.42 22 12A10 10 0 0 0 12 2" />
    </svg>
  );
}

export function BilibiliIcon({ size = 16 }: { size?: number }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M4.903 6.934h14.155s1.467 0 1.467 1.468v9.273s0 1.468-1.467 1.468H4.903s-1.468 0-1.468-1.468V8.402s0-1.468 1.468-1.468" />
      <path d="M14.725 15.839c-.06.27-.35.619-.998.619c-1.178 0-1.707-1.468-1.707-1.468s-.53 1.468-1.707 1.468c-.689 0-.998-.35-.998-.62" />
      <path d="M4.653 4.25A3.913 3.913 0 0 0 .75 8.161v9.763a3.903 3.903 0 0 0 3.903 3.903h.998v.22a1.228 1.228 0 0 0 2.446 0v-.25h7.806v.25a1.227 1.227 0 0 0 2.446 0v-.25h.998a3.903 3.903 0 0 0 3.903-3.903V8.162a3.913 3.913 0 0 0-3.903-3.913zm1.068 7.426l3.843-.779m8.675.779l-3.843-.779M6.12.835L9.534 4.25M17.84.835L14.426 4.25" />
    </svg>
  );
}

export function HeartIcon({ size = 16 }: { size?: number }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="currentColor"
    >
      <path d="m12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5C2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3C19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54z" />
    </svg>
  );
}

const primaryNavItems = [
  { label: "待办事项", icon: CheckSquare, href: "/" },
  { label: "涂鸦推送", icon: PenTool, href: "/sketch-push" },
  { label: "图片推送", icon: Image, href: "/image-push" },
  { label: "自由排版", icon: LayoutGrid, href: "/free-layout" },
  { label: "文本推送", icon: FileText, href: "/text-push" },
  { label: "页面管理", icon: Layers, href: "/page-manager" },
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
          <a
            href="https://github.com/will-17173/zectrix-desktop"
            target="_blank"
            rel="noopener noreferrer"
            className="app-sidebar-github-link"
            aria-label="GitHub 仓库"
          >
            <GithubIcon size={20} />
          </a>
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
