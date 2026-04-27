# Graph Report - src  (2026-04-27)

## Corpus Check
- Corpus is ~19,842 words - fits in a single context window. You may not need a graph.

## Summary
- 199 nodes · 176 edges · 50 communities detected
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Tauri API 函数|Tauri API 函数]]
- [[_COMMUNITY_图片循环任务处理|图片循环任务处理]]
- [[_COMMUNITY_任务 CRUD 操作|任务 CRUD 操作]]
- [[_COMMUNITY_股票推送页面|股票推送页面]]
- [[_COMMUNITY_设置与 API 管理|设置与 API 管理]]
- [[_COMMUNITY_图片模板页面|图片模板页面]]
- [[_COMMUNITY_待办列表页面|待办列表页面]]
- [[_COMMUNITY_图片编辑器对话框|图片编辑器对话框]]
- [[_COMMUNITY_同步状态管理|同步状态管理]]
- [[_COMMUNITY_侧边栏组件|侧边栏组件]]
- [[_COMMUNITY_分析统计|分析统计]]
- [[_COMMUNITY_自由布局页面|自由布局页面]]
- [[_COMMUNITY_图片编辑器|图片编辑器]]
- [[_COMMUNITY_日期工具函数|日期工具函数]]
- [[_COMMUNITY_应用提供者|应用提供者]]
- [[_COMMUNITY_应用工具栏|应用工具栏]]
- [[_COMMUNITY_对话框组件|对话框组件]]
- [[_COMMUNITY_选择器测试|选择器测试]]
- [[_COMMUNITY_提示消息组件|提示消息组件]]
- [[_COMMUNITY_设备管理页面|设备管理页面]]
- [[_COMMUNITY_图片任务卡片|图片任务卡片]]
- [[_COMMUNITY_图片任务列表|图片任务列表]]
- [[_COMMUNITY_图片循环运行器|图片循环运行器]]
- [[_COMMUNITY_草稿页面|草稿页面]]
- [[_COMMUNITY_股票推送测试|股票推送测试]]
- [[_COMMUNITY_同步状态|同步状态]]
- [[_COMMUNITY_文本模板页面|文本模板页面]]
- [[_COMMUNITY_窗口拖拽 Hook|窗口拖拽 Hook]]
- [[_COMMUNITY_分析 Hook|分析 Hook]]
- [[_COMMUNITY_工具函数|工具函数]]
- [[_COMMUNITY_应用根组件|应用根组件]]
- [[_COMMUNITY_应用入口|应用入口]]
- [[_COMMUNITY_Vite 环境类型|Vite 环境类型]]
- [[_COMMUNITY_应用测试|应用测试]]
- [[_COMMUNITY_侧边栏测试|侧边栏测试]]
- [[_COMMUNITY_复选框组件|复选框组件]]
- [[_COMMUNITY_选择器组件|选择器组件]]
- [[_COMMUNITY_标签页组件|标签页组件]]
- [[_COMMUNITY_设备管理测试|设备管理测试]]
- [[_COMMUNITY_自由布局测试|自由布局测试]]
- [[_COMMUNITY_图片任务测试|图片任务测试]]
- [[_COMMUNITY_图片模板测试|图片模板测试]]
- [[_COMMUNITY_插件市场测试|插件市场测试]]
- [[_COMMUNITY_设置页面测试|设置页面测试]]
- [[_COMMUNITY_文本模板测试|文本模板测试]]
- [[_COMMUNITY_待办列表测试|待办列表测试]]
- [[_COMMUNITY_API 契约类型|API 契约类型]]
- [[_COMMUNITY_Zectrix API 客户端|Zectrix API 客户端]]
- [[_COMMUNITY_测试设置|测试设置]]
- [[_COMMUNITY_GA 类型定义|GA 类型定义]]

## God Nodes (most connected - your core abstractions)
1. `getErrorMessage()` - 11 edges
2. `String()` - 6 edges
3. `createEmptyDraft()` - 4 edges
4. `String()` - 4 edges
5. `String()` - 3 edges
6. `handleSave()` - 3 edges
7. `handleDelete()` - 3 edges
8. `handlePush()` - 3 edges
9. `handleCreateLoop()` - 3 edges
10. `handleAdd()` - 3 edges

## Surprising Connections (you probably didn't know these)
- None detected - all connections are within the same source files.

## Communities

### Community 0 - "Tauri API 函数"
Cohesion: 0.04
Nodes (0): 

### Community 1 - "图片循环任务处理"
Cohesion: 0.25
Nodes (13): createEmptyDraft(), getErrorMessage(), handleBuiltinCreateLoop(), handleBuiltinPush(), handleCreateLoop(), handleDelete(), handleDeleteLoopTask(), handleNewPlugin() (+5 more)

### Community 2 - "任务 CRUD 操作"
Cohesion: 0.2
Nodes (0): 

### Community 3 - "股票推送页面"
Cohesion: 0.33
Nodes (7): handleAdd(), handlePush(), handleRemove(), handleStartLoop(), handleStopLoop(), String(), validateCode()

### Community 4 - "设置与 API 管理"
Cohesion: 0.22
Nodes (0): 

### Community 5 - "图片模板页面"
Cohesion: 0.29
Nodes (2): contentTypeIcon(), renderThumbnail()

### Community 6 - "待办列表页面"
Cohesion: 0.29
Nodes (2): handleSubmit(), resetForm()

### Community 7 - "图片编辑器对话框"
Cohesion: 0.43
Nodes (5): getMinDateTime(), handleSave(), handleSelectFolder(), scanFolder(), String()

### Community 8 - "同步状态管理"
Cohesion: 0.33
Nodes (0): 

### Community 9 - "侧边栏组件"
Cohesion: 0.4
Nodes (0): 

### Community 10 - "分析统计"
Cohesion: 0.5
Nodes (0): 

### Community 11 - "自由布局页面"
Cohesion: 1.0
Nodes (2): handlePush(), String()

### Community 12 - "图片编辑器"
Cohesion: 0.67
Nodes (0): 

### Community 13 - "日期工具函数"
Cohesion: 0.67
Nodes (0): 

### Community 14 - "应用提供者"
Cohesion: 1.0
Nodes (0): 

### Community 15 - "应用工具栏"
Cohesion: 1.0
Nodes (0): 

### Community 16 - "对话框组件"
Cohesion: 1.0
Nodes (0): 

### Community 17 - "选择器测试"
Cohesion: 1.0
Nodes (0): 

### Community 18 - "提示消息组件"
Cohesion: 1.0
Nodes (0): 

### Community 19 - "设备管理页面"
Cohesion: 1.0
Nodes (0): 

### Community 20 - "图片任务卡片"
Cohesion: 1.0
Nodes (0): 

### Community 21 - "图片任务列表"
Cohesion: 1.0
Nodes (0): 

### Community 22 - "图片循环运行器"
Cohesion: 1.0
Nodes (0): 

### Community 23 - "草稿页面"
Cohesion: 1.0
Nodes (0): 

### Community 24 - "股票推送测试"
Cohesion: 1.0
Nodes (0): 

### Community 25 - "同步状态"
Cohesion: 1.0
Nodes (0): 

### Community 26 - "文本模板页面"
Cohesion: 1.0
Nodes (0): 

### Community 27 - "窗口拖拽 Hook"
Cohesion: 1.0
Nodes (0): 

### Community 28 - "分析 Hook"
Cohesion: 1.0
Nodes (0): 

### Community 29 - "工具函数"
Cohesion: 1.0
Nodes (0): 

### Community 30 - "应用根组件"
Cohesion: 1.0
Nodes (0): 

### Community 31 - "应用入口"
Cohesion: 1.0
Nodes (0): 

### Community 32 - "Vite 环境类型"
Cohesion: 1.0
Nodes (0): 

### Community 33 - "应用测试"
Cohesion: 1.0
Nodes (0): 

### Community 34 - "侧边栏测试"
Cohesion: 1.0
Nodes (0): 

### Community 35 - "复选框组件"
Cohesion: 1.0
Nodes (0): 

### Community 36 - "选择器组件"
Cohesion: 1.0
Nodes (0): 

### Community 37 - "标签页组件"
Cohesion: 1.0
Nodes (0): 

### Community 38 - "设备管理测试"
Cohesion: 1.0
Nodes (0): 

### Community 39 - "自由布局测试"
Cohesion: 1.0
Nodes (0): 

### Community 40 - "图片任务测试"
Cohesion: 1.0
Nodes (0): 

### Community 41 - "图片模板测试"
Cohesion: 1.0
Nodes (0): 

### Community 42 - "插件市场测试"
Cohesion: 1.0
Nodes (0): 

### Community 43 - "设置页面测试"
Cohesion: 1.0
Nodes (0): 

### Community 44 - "文本模板测试"
Cohesion: 1.0
Nodes (0): 

### Community 45 - "待办列表测试"
Cohesion: 1.0
Nodes (0): 

### Community 46 - "API 契约类型"
Cohesion: 1.0
Nodes (0): 

### Community 47 - "Zectrix API 客户端"
Cohesion: 1.0
Nodes (0): 

### Community 48 - "测试设置"
Cohesion: 1.0
Nodes (0): 

### Community 49 - "GA 类型定义"
Cohesion: 1.0
Nodes (0): 

## Knowledge Gaps
- **Thin community `应用提供者`** (2 nodes): `Providers()`, `providers.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `应用工具栏`** (2 nodes): `AppToolbar()`, `app-toolbar.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `对话框组件`** (2 nodes): `DialogHeader()`, `dialog.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `选择器测试`** (2 nodes): `TestSelect()`, `select.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `提示消息组件`** (2 nodes): `toast.tsx`, `Toaster()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `设备管理页面`** (2 nodes): `DeviceManagementPage()`, `device-management-page.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `图片任务卡片`** (2 nodes): `ImageLoopTaskCard()`, `image-loop-task-card.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `图片任务列表`** (2 nodes): `ImageLoopTaskList()`, `image-loop-task-list.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `图片循环运行器`** (2 nodes): `use-image-loop-runner.ts`, `useImageLoopRunner()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `草稿页面`** (2 nodes): `handlePush()`, `sketch-page.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `股票推送测试`** (2 nodes): `stock-push-page.test.tsx`, `createDeferred()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `同步状态`** (2 nodes): `sync-status.ts`, `syncStateLabel()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `文本模板页面`** (2 nodes): `text-templates-page.tsx`, `TextTemplatesPage()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `窗口拖拽 Hook`** (2 nodes): `use-window-drag.ts`, `useWindowDrag()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `分析 Hook`** (2 nodes): `useAnalytics.ts`, `useAnalytics()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `工具函数`** (2 nodes): `utils.ts`, `cn()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `应用根组件`** (1 nodes): `App.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `应用入口`** (1 nodes): `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Vite 环境类型`** (1 nodes): `vite-env.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `应用测试`** (1 nodes): `App.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `侧边栏测试`** (1 nodes): `app-sidebar.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `复选框组件`** (1 nodes): `checkbox.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `选择器组件`** (1 nodes): `select.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `标签页组件`** (1 nodes): `tabs.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `设备管理测试`** (1 nodes): `device-management-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `自由布局测试`** (1 nodes): `free-layout-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `图片任务测试`** (1 nodes): `image-loop-task-list.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `图片模板测试`** (1 nodes): `image-templates-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `插件市场测试`** (1 nodes): `plugin-market-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `设置页面测试`** (1 nodes): `settings-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `文本模板测试`** (1 nodes): `text-templates-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `待办列表测试`** (1 nodes): `todo-list-page.test.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `API 契约类型`** (1 nodes): `contracts.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Zectrix API 客户端`** (1 nodes): `zectrix-client.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `测试设置`** (1 nodes): `setup.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `GA 类型定义`** (1 nodes): `gtag.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Should `Tauri API 函数` be split into smaller, more focused modules?**
  _Cohesion score 0.04 - nodes in this community are weakly interconnected._