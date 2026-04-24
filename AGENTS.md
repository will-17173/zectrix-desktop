# Repository Guidelines

## Project Structure & Module Organization
`src/` contains the React frontend. Key areas are `src/app/` for app bootstrap and routing, `src/components/` for reusable UI, `src/features/` for page-level features, and `src/lib/` for shared helpers and Tauri bindings. Frontend tests live next to the code as `*.test.tsx`. Rust backend code is under `src-tauri/`. Static assets are in `assets/` and `public/`. Keep generated output such as `dist/` and `src/graphify-out/cache/` out of manual edits.

## Build, Test, and Development Commands
- `pnpm install` - install frontend dependencies.
- `pnpm tauri dev` - run the desktop app in development mode with Vite hot reload.
- `pnpm build` - type-check and build the frontend bundle.
- `pnpm tauri build` - build the production desktop package.
- `pnpm vitest run` - run the frontend test suite once.
- `pnpm vitest watch` - rerun frontend tests during development.
- `cargo test --manifest-path src-tauri/Cargo.toml` - run Rust tests for the Tauri backend.

## Coding Style & Naming Conventions
Follow the existing TypeScript/React style in the repo: 2-space indentation, double quotes, semicolons, and trailing commas where the surrounding file uses them. Use `PascalCase` for React components and page files, `camelCase` for hooks and helpers, and `*.test.tsx` for tests. Prefer colocated feature modules under `src/features/<area>/` and reuse the shared UI wrappers in `src/components/ui/` instead of adding one-off patterns. **Do not use native `<select>` elements — always use the shadcn/ui Select component from `src/components/ui/select.tsx`.**

## Testing Guidelines
Use Vitest with Testing Library for UI behavior. Mock Tauri-facing helpers in `src/lib/tauri` when tests need app state or backend calls. Keep tests focused on visible behavior: rendering, interactions, and state updates. Name tests by behavior, for example `renders the sidebar footer group` or `creates a todo from the collapsible form`.

## Commit & Pull Request Guidelines
The history uses conventional prefixes such as `feat:`, `fix:`, and `docs:`. Keep commit messages short and scoped to one change. Pull requests should include a clear summary, the commands used to verify the change, and screenshots or screen recordings for UI work. Link related issues when relevant.

## Configuration & Data Notes
The app stores user data outside the repo in the user profile, not in source control. Avoid committing local machine artifacts, personal data, or regenerated bundles unless the change explicitly targets them.

## Agent-Specific Instructions
Do not automatically invoke the `using-superpowers` skill unless the user explicitly asks to use it.
