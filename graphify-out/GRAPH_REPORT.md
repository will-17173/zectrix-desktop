# Graph Report - src  (2026-04-23)

## Corpus Check
- Corpus is ~6,821 words - fits in a single context window. You may not need a graph.

## Summary
- 137 nodes · 93 edges · 50 communities detected
- Extraction: 78% EXTRACTED · 22% INFERRED · 0% AMBIGUOUS · INFERRED: 20 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Tauri API Commands|Tauri API Commands]]
- [[_COMMUNITY_Push & Sync Handlers|Push & Sync Handlers]]
- [[_COMMUNITY_Todo Page Handlers|Todo Page Handlers]]
- [[_COMMUNITY_App Shell & Providers|App Shell & Providers]]
- [[_COMMUNITY_Data Types & Records|Data Types & Records]]
- [[_COMMUNITY_Settings Page Handlers|Settings Page Handlers]]
- [[_COMMUNITY_Shell Layout Components|Shell Layout Components]]
- [[_COMMUNITY_Radix UI Wrappers|Radix UI Wrappers]]
- [[_COMMUNITY_App Sync Logic|App Sync Logic]]
- [[_COMMUNITY_Image Editor Dialog|Image Editor Dialog]]
- [[_COMMUNITY_Image Templates Page|Image Templates Page]]
- [[_COMMUNITY_Date Utilities|Date Utilities]]
- [[_COMMUNITY_Device Management|Device Management]]
- [[_COMMUNITY_Image Workflow|Image Workflow]]
- [[_COMMUNITY_React Brand Assets|React Brand Assets]]
- [[_COMMUNITY_Providers Wrapper|Providers Wrapper]]
- [[_COMMUNITY_Sidebar Component|Sidebar Component]]
- [[_COMMUNITY_Toolbar Component|Toolbar Component]]
- [[_COMMUNITY_Dialog Component|Dialog Component]]
- [[_COMMUNITY_Select Test|Select Test]]
- [[_COMMUNITY_Device Management Page|Device Management Page]]
- [[_COMMUNITY_Sync Status Module|Sync Status Module]]
- [[_COMMUNITY_Text Templates Page|Text Templates Page]]
- [[_COMMUNITY_Window Drag Hook|Window Drag Hook]]
- [[_COMMUNITY_Utility Functions|Utility Functions]]
- [[_COMMUNITY_Src app tsx|Src app tsx]]
- [[_COMMUNITY_Src main tsx|Src main tsx]]
- [[_COMMUNITY_Src vite env d ts|Src vite env d ts]]
- [[_COMMUNITY_Src app app test tsx|Src app app test tsx]]
- [[_COMMUNITY_Module 29|Module 29]]
- [[_COMMUNITY_Src components ui select tsx|Src components ui select tsx]]
- [[_COMMUNITY_Module 31|Module 31]]
- [[_COMMUNITY_Module 32|Module 32]]
- [[_COMMUNITY_Module 33|Module 33]]
- [[_COMMUNITY_Module 34|Module 34]]
- [[_COMMUNITY_Module 35|Module 35]]
- [[_COMMUNITY_Src lib api contracts ts|Src lib api contracts ts]]
- [[_COMMUNITY_Src lib api zectrix client ts|Src lib api zectrix client ts]]
- [[_COMMUNITY_Src test setup ts|Src test setup ts]]
- [[_COMMUNITY_Module 39|Module 39]]
- [[_COMMUNITY_Module 40|Module 40]]
- [[_COMMUNITY_Settings page settings page|Settings page settings page]]
- [[_COMMUNITY_Settings page mask api key|Settings page mask api key]]
- [[_COMMUNITY_Sync status sync state label|Sync status sync state label]]
- [[_COMMUNITY_Module 44|Module 44]]
- [[_COMMUNITY_Module 45|Module 45]]
- [[_COMMUNITY_Module 46|Module 46]]
- [[_COMMUNITY_Module 47|Module 47]]
- [[_COMMUNITY_Utils cn function|Utils cn function]]
- [[_COMMUNITY_Api contract api response|Api contract api response]]

## God Nodes (most connected - your core abstractions)
1. `Tauri API Layer` - 9 edges
2. `App Component` - 6 edges
3. `App Test Suite` - 3 edges
4. `App Sidebar Component` - 3 edges
5. `Select UI Primitives` - 3 edges
6. `handlePush Function` - 3 edges
7. `TodoListPage Component` - 3 edges
8. `BootstrapState Type` - 3 edges
9. `Application Entry Point` - 2 edges
10. `Providers Component` - 2 edges

## Surprising Connections (you probably didn't know these)
- `Providers Component` --shares_data_with--> `App Component`  [INFERRED]
  src/app/providers.tsx → src/app/App.tsx
- `App Sidebar Component` --semantically_similar_to--> `App Toolbar Component`  [INFERRED] [semantically similar]
  src/components/layout/app-sidebar.tsx → src/components/layout/app-toolbar.tsx
- `Dialog UI Primitives` --semantically_similar_to--> `Select UI Primitives`  [INFERRED] [semantically similar]
  src/components/ui/dialog.tsx → src/components/ui/select.tsx
- `handleSave Function` --calls--> `Tauri API Layer`  [INFERRED]
  src/features/images/image-templates-page.tsx → src/lib/tauri.ts
- `handlePush Function` --calls--> `Tauri API Layer`  [INFERRED]
  src/features/images/image-templates-page.tsx → src/lib/tauri.ts

## Hyperedges (group relationships)
- **Shell Chrome Layout Components** — app_App, app_sidebar_AppSidebar, app_toolbar_AppToolbar [EXTRACTED 1.00]
- **Radix UI Primitive Wrappers** — ui_dialog, ui_select, radix_primitive_wrapper [EXTRACTED 1.00]
- **Sync State Flow** — app_App, sync_SyncState, sync_feedback_mechanism [EXTRACTED 1.00]
- **Push to Device Pattern** — image_templates_page_handle_push, text_templates_page_handle_push, todo_list_page_handle_push, tauri_tauri_api_layer [INFERRED 0.85]
- **Bootstrap State Data Aggregation** — tauri_bootstrap_state, tauri_api_key_record, tauri_device_record, tauri_todo_record [EXTRACTED 1.00]
- **Page Components with Device Selection** — settings_page_settings_page, todo_list_page_todo_list_page, image_templates_page_image_templates_page, text_templates_page_text_templates_page [INFERRED 0.75]

## Communities

### Community 0 - "Tauri API Commands"
Cohesion: 0.11
Nodes (0): 

### Community 1 - "Push & Sync Handlers"
Cohesion: 0.15
Nodes (14): handlePush Function, handleSave Function, handleAddApiKey Function, handleAddDevice Function, MAC Address Validation Pattern, SyncState Type, pushTodoToDevice Function, Tauri API Layer (+6 more)

### Community 2 - "Todo Page Handlers"
Cohesion: 0.25
Nodes (0): 

### Community 3 - "App Shell & Providers"
Cohesion: 0.32
Nodes (8): App Component, App Test Suite, Application Entry Point, Providers Component, Bootstrap State, Shell Chrome Layout Pattern, Sync State Type, Sync Feedback Mechanism

### Community 4 - "Data Types & Records"
Cohesion: 0.25
Nodes (8): ApiDevice Type, ApiKeyRecord Type, BootstrapState Type, DeviceRecord Type, TodoRecord Type, Test Setup Polyfills, TodoListPage Component, Visible Todos Filter Logic

### Community 5 - "Settings Page Handlers"
Cohesion: 0.29
Nodes (0): 

### Community 6 - "Shell Layout Components"
Cohesion: 0.67
Nodes (4): App Sidebar Component, App Sidebar Test Suite, App Toolbar Component, Window Drag Hook

### Community 7 - "Radix UI Wrappers"
Cohesion: 0.67
Nodes (4): Radix Primitive Wrapper Pattern, Dialog UI Primitives, Select UI Primitives, Select Test Suite

### Community 8 - "App Sync Logic"
Cohesion: 0.67
Nodes (0): 

### Community 9 - "Image Editor Dialog"
Cohesion: 0.67
Nodes (0): 

### Community 10 - "Image Templates Page"
Cohesion: 0.67
Nodes (0): 

### Community 11 - "Date Utilities"
Cohesion: 0.67
Nodes (0): 

### Community 12 - "Device Management"
Cohesion: 1.0
Nodes (3): MAC Address Validation Pattern, Device Management Page, Device Management Test Suite

### Community 13 - "Image Workflow"
Cohesion: 0.67
Nodes (3): Image Crop Workflow, Image Editor Dialog, Image Templates Test Suite

### Community 14 - "React Brand Assets"
Cohesion: 0.67
Nodes (3): React Atom Symbol Design, React Framework, React Logo

### Community 15 - "Providers Wrapper"
Cohesion: 1.0
Nodes (0): 

### Community 16 - "Sidebar Component"
Cohesion: 1.0
Nodes (0): 

### Community 17 - "Toolbar Component"
Cohesion: 1.0
Nodes (0): 

### Community 18 - "Dialog Component"
Cohesion: 1.0
Nodes (0): 

### Community 19 - "Select Test"
Cohesion: 1.0
Nodes (0): 

### Community 20 - "Device Management Page"
Cohesion: 1.0
Nodes (0): 

### Community 21 - "Sync Status Module"
Cohesion: 1.0
Nodes (0): 

### Community 22 - "Text Templates Page"
Cohesion: 1.0
Nodes (0): 

### Community 23 - "Window Drag Hook"
Cohesion: 1.0
Nodes (0): 

### Community 24 - "Utility Functions"
Cohesion: 1.0
Nodes (0): 

### Community 25 - "Src app tsx"
Cohesion: 1.0
Nodes (0): 

### Community 26 - "Src main tsx"
Cohesion: 1.0
Nodes (0): 

### Community 27 - "Src vite env d ts"
Cohesion: 1.0
Nodes (0): 

### Community 28 - "Src app app test tsx"
Cohesion: 1.0
Nodes (0): 

### Community 29 - "Module 29"
Cohesion: 1.0
Nodes (0): 

### Community 30 - "Src components ui select tsx"
Cohesion: 1.0
Nodes (0): 

### Community 31 - "Module 31"
Cohesion: 1.0
Nodes (0): 

### Community 32 - "Module 32"
Cohesion: 1.0
Nodes (0): 

### Community 33 - "Module 33"
Cohesion: 1.0
Nodes (0): 

### Community 34 - "Module 34"
Cohesion: 1.0
Nodes (0): 

### Community 35 - "Module 35"
Cohesion: 1.0
Nodes (0): 

### Community 36 - "Src lib api contracts ts"
Cohesion: 1.0
Nodes (0): 

### Community 37 - "Src lib api zectrix client ts"
Cohesion: 1.0
Nodes (0): 

### Community 38 - "Src test setup ts"
Cohesion: 1.0
Nodes (0): 

### Community 39 - "Module 39"
Cohesion: 1.0
Nodes (1): ImageTemplatesPage Component

### Community 40 - "Module 40"
Cohesion: 1.0
Nodes (1): ImageTemplateRecord Type

### Community 41 - "Settings page settings page"
Cohesion: 1.0
Nodes (1): SettingsPage Component

### Community 42 - "Settings page mask api key"
Cohesion: 1.0
Nodes (1): maskApiKey Function

### Community 43 - "Sync status sync state label"
Cohesion: 1.0
Nodes (1): syncStateLabel Function

### Community 44 - "Module 44"
Cohesion: 1.0
Nodes (1): TextTemplatesPage Component

### Community 45 - "Module 45"
Cohesion: 1.0
Nodes (1): TextTemplateRecord Type

### Community 46 - "Module 46"
Cohesion: 1.0
Nodes (1): formatDeadline Function

### Community 47 - "Module 47"
Cohesion: 1.0
Nodes (1): useWindowDrag Hook

### Community 48 - "Utils cn function"
Cohesion: 1.0
Nodes (1): cn Utility Function

### Community 49 - "Api contract api response"
Cohesion: 1.0
Nodes (1): ApiResponse Type

## Knowledge Gaps
- **31 isolated node(s):** `App Sidebar Test Suite`, `Select Test Suite`, `Image Templates Test Suite`, `Bootstrap State`, `Shell Chrome Layout Pattern` (+26 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Providers Wrapper`** (2 nodes): `Providers()`, `providers.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Sidebar Component`** (2 nodes): `SidebarLink()`, `app-sidebar.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Toolbar Component`** (2 nodes): `AppToolbar()`, `app-toolbar.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dialog Component`** (2 nodes): `DialogHeader()`, `dialog.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Select Test`** (2 nodes): `TestSelect()`, `select.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Device Management Page`** (2 nodes): `DeviceManagementPage()`, `device-management-page.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Sync Status Module`** (2 nodes): `sync-status.ts`, `syncStateLabel()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Text Templates Page`** (2 nodes): `text-templates-page.tsx`, `TextTemplatesPage()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Window Drag Hook`** (2 nodes): `use-window-drag.ts`, `useWindowDrag()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Utility Functions`** (2 nodes): `utils.ts`, `cn()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Src app tsx`** (1 nodes): `App.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Src main tsx`** (1 nodes): `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Src vite env d ts`** (1 nodes): `vite-env.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Src app app test tsx`** (1 nodes): `App.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 29`** (1 nodes): `app-sidebar.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Src components ui select tsx`** (1 nodes): `select.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 31`** (1 nodes): `device-management-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 32`** (1 nodes): `image-templates-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 33`** (1 nodes): `settings-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 34`** (1 nodes): `text-templates-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 35`** (1 nodes): `todo-list-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Src lib api contracts ts`** (1 nodes): `contracts.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Src lib api zectrix client ts`** (1 nodes): `zectrix-client.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Src test setup ts`** (1 nodes): `setup.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 39`** (1 nodes): `ImageTemplatesPage Component`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 40`** (1 nodes): `ImageTemplateRecord Type`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Settings page settings page`** (1 nodes): `SettingsPage Component`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Settings page mask api key`** (1 nodes): `maskApiKey Function`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Sync status sync state label`** (1 nodes): `syncStateLabel Function`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 44`** (1 nodes): `TextTemplatesPage Component`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 45`** (1 nodes): `TextTemplateRecord Type`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 46`** (1 nodes): `formatDeadline Function`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Module 47`** (1 nodes): `useWindowDrag Hook`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Utils cn function`** (1 nodes): `cn Utility Function`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Api contract api response`** (1 nodes): `ApiResponse Type`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Are the 9 inferred relationships involving `Tauri API Layer` (e.g. with `handleSave Function` and `handlePush Function`) actually correct?**
  _`Tauri API Layer` has 9 INFERRED edges - model-reasoned connections that need verification._
- **What connects `App Sidebar Test Suite`, `Select Test Suite`, `Image Templates Test Suite` to the rest of the system?**
  _31 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Tauri API Commands` be split into smaller, more focused modules?**
  _Cohesion score 0.11 - nodes in this community are weakly interconnected._