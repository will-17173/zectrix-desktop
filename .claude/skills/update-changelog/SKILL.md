---
name: update-changelog
description: 更新 CHANGELOG.md 并同步版本号到所有配置文件。自动提取从上一个 tag 到当前 HEAD 的所有 commit，按类型分类整理成中文格式的版本说明。需要传入版本号参数，会同时更新 package.json、Cargo.toml、tauri.conf.json 中的版本号。当用户说"更新 changelog"、"生成版本说明"、"准备发布"、"写 changelog"、"更新版本"并指定版本号时触发。
---

# 更新 CHANGELOG

自动提取 git commit 历史，生成符合 Keep a Changelog 格式的中文版本说明。

**用法：** 传入版本号作为参数，如 `0.3.0`、`v1.0.0` 等。

## 工作流程

1. **获取上一个 tag**
   ```bash
   git describe --tags --abbrev=0
   ```
   如果没有 tag，则从第一个 commit 开始。

2. **获取 commit 历史**
   ```bash
   git log <last-tag>..HEAD --oneline --no-decorate
   ```

3. **分类整理 commit**

   根据 commit message 前缀分类：

   | 前缀 | 中文分类 |
   |------|----------|
   | `feat:` | 新增 |
   | `fix:` | 修复 |
   | `docs:` | 文档 |
   | `style:` | 样式 |
   | `refactor:` | 变更 |
   | `perf:` | 性能 |
   | `test:` | 测试 |
   | `chore:` | 其他 |
   | `build:` | 构建 |
   | `ci:` | CI |

   对于没有前缀的 commit，根据内容判断：
   - 包含 "add"、"新增"、"实现" → 新增
   - 包含 "fix"、"修复"、"解决" → 修复
   - 包含 "update"、"更新"、"change" → 变更
   - 包含 "remove"、"删除"、"移除" → 移除
   - 其他 → 其他

4. **获取版本号**

   使用用户传入的版本号参数。如果传入的是 `v0.3.0` 格式，去掉 `v` 前缀使用 `0.3.0`。

5. **生成日期**

   使用当前日期，格式：YYYY-MM-DD

6. **格式化输出**

   生成以下格式的条目：

   ```markdown
   ## [版本号] - YYYY-MM-DD

   ### 新增
   - 功能描述（去除 feat: 前缀，保持简洁）

   ### 修复
   - 修复描述

   ### 变更
   - 变更描述
   ```

   如果某个分类没有内容，则跳过该分类。

7. **插入 CHANGELOG.md**

   将生成的条目插入到 `# 更新日志` 标题之后，第一个版本条目之前。

8. **同步版本号到代码**

   更新以下文件中的版本号：

   - `package.json` — 更新 `"version"` 字段
   - `src-tauri/Cargo.toml` — 更新 `version = "x.x.x"` 行
   - `src-tauri/tauri.conf.json` — 更新 `"version"` 字段

   使用 Edit 工具替换旧的版本号为新的版本号。

9. **更新 README.md 核心特性**

   如果本次版本有 `feat:` 类型的 commit，将新功能总结添加到 README.md 的「核心特性」部分。

   **规则：**
   - 只处理 `feat:` 前缀的 commit，其他类型跳过
   - 如果没有 `feat:` commit，则不修改 README.md
   - 将多个相关 commit 合并总结为一条特性描述
   - 描述要简洁概括，不要原样复制 commit message
   - 添加到「核心特性」列表的合适位置（按功能相关性排序）
   - 保持与现有特性描述风格一致（使用 `**粗体** — 说明` 格式）

   **示例：**
   ```
   # 如果有这些 commits：
   feat: add calendar sync types and invoke functions to tauri.ts
   feat: add CalendarSyncPanel component with tests
   feat: move calendar sync to todo page and fix permission bug

   # 在 README 核心特性中添加：
   - **Apple 日历同步** — 待办事项可同步至 macOS 日历事件或提醒事项，支持双向同步（仅 macOS）
   ```

## 输出要求

- commit 描述要简洁，去掉前缀（如 `feat:`）
- 多个相似的 commit 可以合并为一条
- 使用中文描述
- 保持 CHANGELOG.md 现有的格式风格

## 示例

**调用方式：**
```
/update-changelog 0.3.0
```

**执行结果：**

1. CHANGELOG.md 添加新版本条目：
```markdown
## [0.3.0] - 2025-04-26

### 新增
- GitHub Actions 监控插件

### 修复
- 插件配置持久化问题
```

2. 更新以下文件的版本号为 `0.3.0`：
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`