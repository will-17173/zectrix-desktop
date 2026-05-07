# 更新日志

本项目的所有重要变更都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [0.2.5] - 2026-05-07

### 新增
- macOS 日历同步功能（支持日历事件和提醒事项）
- CalendarBridge Swift CLI 实现 EventKit 交互
- CalendarSyncPanel 组件及设置对话框
- 待办页面添加"Apple日历同步"按钮（仅 macOS 可见）
- 日历同步配置管理（启用/禁用、目标类型、同步方向）
- TodoRecord 添加日历同步相关字段

### 修复
- macOS 14+ 日历权限请求逻辑（区分事件和提醒事项权限）
- Info.plist 日历权限描述配置

### 测试
- CalendarSync 配置和 TodoRecord 字段的 Rust 单元测试

### 文档
- macOS 日历同步设计规范

## [0.2.4] - 2026-04-28

### 新增
- 多市场股票代码支持（港股、美股、A股）
- StockCode 结构体及 parse_stock_input 解析函数
- 股票推送文本添加市场标签（[港]/[美]）
- 股票行情数据拉取支持腾讯多市场接口

### 变更
- 内置插件按插件拆分为独立模块
- 前端股票代码展示适配多市场格式

### 文档
- 插件模块化与多市场股票支持设计规范
- macOS Gatekeeper 绕过说明添加到 README

## [0.2.3] - 2026-04-27

### 新增
- GitHub Trending 插件
- B站UP主信息插件
- 插件分类功能（分类按钮组和筛选）
- 插件生成器分类支持
- GitHub Actions 插件配置对话框
- Zectrix Cloud API skill
- ComfyUI 随机种子选项

### 修复
- 分类 tab 边框自适应 button group 宽度

### 文档
- 插件分类设计规范
- QuickJS 环境文档
- README 更新截图

### 其他
- graphify 输出更新

## [0.2.2] - 2026-04-26

### 新增
- ComfyUI 图片生成插件

## [0.2.1] - 2025-04-26

### 新增
- GitHub Actions 监控插件和 release changelog 自动化

## [0.2.0] - 2025-04-26

### 新增
- 插件市场功能 - 浏览和安装内置插件
- 股票推送页面 - 在墨水屏设备上显示股票信息

### 修复
- Rust 编译器警告

## [0.1.0] - 2025-04-01

### 新增
- 初始版本
- 待办管理，采用本地优先架构
- 墨水屏设备内容推送支持
- 模板管理
- 通过 keyring 管理 API 密钥