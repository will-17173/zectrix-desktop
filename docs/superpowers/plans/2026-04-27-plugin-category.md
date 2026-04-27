# 内置插件分类实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为内置插件新增子分类功能，使用 ButtonGroup 切换分类，并更新 plugin-generator skill 支持分类输入。

**Architecture:** 在 Rust 数据模型 `BuiltinPlugin` 中新增 `category` 字段，前端根据此字段渲染分类按钮组并过滤插件列表。

**Tech Stack:** Rust (Tauri), TypeScript, React, shadcn/ui, Tailwind CSS

---

## File Structure

| File | Responsibility |
|------|-----------------|
| `src-tauri/src/builtin_plugins.rs` | Rust 数据模型 `BuiltinPlugin` 新增 `category` 字段；为每个插件设置分类 |
| `src/lib/tauri.ts` | TypeScript 类型 `BuiltinPlugin` 新增 `category?` 字段 |
| `src/features/plugins/plugin-market-page.tsx` | 新增 ButtonGroup 分类选择器、过滤逻辑、状态管理 |
| `.claude/skills/plugin-generator/SKILL.md` | 更新 skill 文档，新增分类输入和生成步骤 |

---

### Task 1: 修改 Rust BuiltinPlugin 结构体新增 category 字段

**Files:**
- Modify: `src-tauri/src/builtin_plugins.rs:22-34`

- [ ] **Step 1: 在 BuiltinPlugin struct 中新增 category 字段**

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinPlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub code: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub config: Vec<PluginConfigOption>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub supports_loop: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub category: String,  // 新增
}
```

- [ ] **Step 2: 更新 BuiltinPlugin 的 Default 实现**

```rust
impl Default for BuiltinPlugin {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            code: String::new(),
            config: vec![],
            supports_loop: true,
            category: String::new(),  // 新增
        }
    }
}
```

- [ ] **Step 3: 运行 cargo check 验证编译**

Run: `cd /Volumes/T7/Code/zectrix-note-4/src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 4: Commit**

```bash
cd /Volumes/T7/Code/zectrix-note-4
git add src-tauri/src/builtin_plugins.rs
git commit -m "feat: add category field to BuiltinPlugin struct"
```

---

### Task 2: 为现有插件设置分类

**Files:**
- Modify: `src-tauri/src/builtin_plugins.rs:58-422`

- [ ] **Step 1: 为 comfyui-image 插件添加 category: "AI"**

在 `list_builtin_plugins()` 函数中，找到 `comfyui-image` 插件的 `BuiltinPlugin` 定义，在 `code:` 字段前添加：

```rust
BuiltinPlugin {
    id: "comfyui-image".to_string(),
    name: "ComfyUI 生图".to_string(),
    description: "调用 ComfyUI 生成图片并推送到设备".to_string(),
    category: "AI".to_string(),  // 新增
    config: vec![...],
    code: r#"..."#.to_string(),
    supports_loop: false,
    ..Default::default()
},
```

- [ ] **Step 2: 为 cat-random 插件添加 category: "图片"**

```rust
BuiltinPlugin {
    id: "cat-random".to_string(),
    name: "随机显示猫猫".to_string(),
    description: "随机获取一张猫猫图片并推送到设备".to_string(),
    category: "图片".to_string(),  // 新增
    code: r#"..."#.to_string(),
    config: vec![],
    ..Default::default()
},
```

- [ ] **Step 3: 为 dog-random 插件添加 category: "图片"**

```rust
BuiltinPlugin {
    id: "dog-random".to_string(),
    name: "随机显示狗狗".to_string(),
    description: "随机获取一张狗狗图片并推送到设备".to_string(),
    category: "图片".to_string(),  // 新增
    code: r#"..."#.to_string(),
    config: vec![],
    ..Default::default()
},
```

- [ ] **Step 4: 为 duck-random 插件添加 category: "图片"**

```rust
BuiltinPlugin {
    id: "duck-random".to_string(),
    name: "随机显示鸭子".to_string(),
    description: "随机获取一张鸭子图片并推送到设备".to_string(),
    category: "图片".to_string(),  // 新增
    code: r#"..."#.to_string(),
    config: vec![],
    ..Default::default()
},
```

- [ ] **Step 5: 为 waifu-random 插件添加 category: "图片"**

```rust
BuiltinPlugin {
    id: "waifu-random".to_string(),
    name: "随机显示 Waifu".to_string(),
    description: "随机获取一张动漫图片并推送到设备".to_string(),
    category: "图片".to_string(),  // 新增
    config: vec![...],
    code: r#"..."#.to_string(),
    ..Default::default()
},
```

- [ ] **Step 6: 为 text-to-qrcode 插件添加 category: "工具"**

```rust
BuiltinPlugin {
    id: "text-to-qrcode".to_string(),
    name: "文本转二维码".to_string(),
    description: "将输入的文本转换为二维码图片".to_string(),
    category: "工具".to_string(),  // 新增
    config: vec![...],
    code: r#"..."#.to_string(),
    ..Default::default()
},
```

- [ ] **Step 7: 为 poetry-random 插件添加 category: "文学"**

```rust
BuiltinPlugin {
    id: "poetry-random".to_string(),
    name: "随机古诗词".to_string(),
    description: "随机获取一首古诗词并推送到设备".to_string(),
    category: "文学".to_string(),  // 新增
    config: vec![],
    code: r#"..."#.to_string(),
    ..Default::default()
},
```

- [ ] **Step 8: 为 github-actions 插件添加 category: "编程"**

```rust
BuiltinPlugin {
    id: "github-actions".to_string(),
    name: "GitHub Actions 监控".to_string(),
    description: "监控指定仓库的 GitHub Actions 运行状态".to_string(),
    category: "编程".to_string(),  // 新增
    config: vec![...],
    code: r#"..."#.to_string(),
    ..Default::default()
},
```

- [ ] **Step 9: 运行 cargo check 验证编译**

Run: `cd /Volumes/T7/Code/zectrix-note-4/src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 10: Commit**

```bash
cd /Volumes/T7/Code/zectrix-note-4
git add src-tauri/src/builtin_plugins.rs
git commit -m "feat: add category to all builtin plugins"
```

---

### Task 3: 修改 TypeScript BuiltinPlugin 类型

**Files:**
- Modify: `src/lib/tauri.ts:313-320`

- [ ] **Step 1: 在 BuiltinPlugin 类型中新增 category 字段**

```typescript
export type BuiltinPlugin = {
  id: string;
  name: string;
  description: string;
  code: string;
  config?: PluginConfigOption[];
  supportsLoop?: boolean;
  category?: string;  // 新增
};
```

- [ ] **Step 2: Commit**

```bash
cd /Volumes/T7/Code/zectrix-note-4
git add src/lib/tauri.ts
git commit -m "feat: add category field to BuiltinPlugin TypeScript type"
```

---

### Task 4: 修改前端 plugin-market-page.tsx 新增 ButtonGroup 和过滤逻辑

**Files:**
- Modify: `src/features/plugins/plugin-market-page.tsx`

- [ ] **Step 1: 新增 useState 用于 selectedCategory**

在 `PluginMarketPage` 组件内，其他 useState 附近添加：

```tsx
const [selectedCategory, setSelectedCategory] = useState("");
```

- [ ] **Step 2: 新增 useMemo 提取分类列表**

在组件内添加：

```tsx
const categories = useMemo(() => {
  const cats = new Set<string>();
  builtinPlugins.forEach(p => { if (p.category) cats.add(p.category); });
  return [
    { key: "", label: "全部" },
    ...Array.from(cats).sort().map(c => ({ key: c, label: c }))
  ];
}, [builtinPlugins]);
```

- [ ] **Step 3: 新增 useMemo 过滤插件列表**

```tsx
const filteredPlugins = useMemo(() => {
  if (!selectedCategory) return builtinPlugins;
  return builtinPlugins.filter(p => p.category === selectedCategory);
}, [builtinPlugins, selectedCategory]);
```

- [ ] **Step 4: 在 TabsContent value="builtin" 内添加 ButtonGroup**

找到 `<TabsContent value="builtin" ...>`，在 `<p className="mb-4 text-sm text-gray-500">` 之前添加 ButtonGroup：

```tsx
<TabsContent value="builtin" className="rounded-xl border border-blue-200 bg-gradient-to-br from-blue-50/50 to-white p-4 shadow-sm">
  <p className="mb-4 text-sm text-gray-500">
    内置插件数量正在开发中，欢迎给作者 Bilibili up{' '}
    <a href="https://space.bilibili.com/328381287" target="_blank" rel="noopener noreferrer" className="text-blue-600 hover:underline">@Terminator-AI</a>{' '}私信提出开发需求。
  </p>

  {/* 分类按钮组 */}
  <div className="flex items-center rounded-md border border-gray-300 overflow-hidden mb-3">
    {categories.map((cat, index) => (
      <button
        key={cat.key}
        onClick={() => setSelectedCategory(cat.key)}
        className={`px-3 py-1.5 text-xs font-medium border-r border-gray-300 first:rounded-l-md last:rounded-r-md last:border-r-0 ${
          selectedCategory === cat.key
            ? "bg-blue-500 text-white shadow-sm"
            : "bg-white text-gray-600 hover:bg-gray-50"
        }`}
      >
        {cat.label}
      </button>
    ))}
  </div>

  {filteredPlugins.length === 0 ? (
    <p className="text-sm text-gray-500">暂无内置插件</p>
  ) : (
    <div className="grid gap-3 lg:grid-cols-2">
      {filteredPlugins.map((plugin) => (
        <PluginCard key={plugin.id} ... />
      ))}
    </div>
  )}
</TabsContent>
```

注意：需要将原来对 `builtinPlugins` 的引用改为 `filteredPlugins`。

- [ ] **Step 5: 运行前端检查确保无 TypeScript 错误**

Run: `cd /Volumes/T7/Code/zectrix-note-4 && npx tsc --noEmit`
Expected: 无 TypeScript 错误

- [ ] **Step 6: 运行 pnpm vitest run src/features/plugins/plugin-market-page.test.tsx 验证测试通过**

Run: `cd /Volumes/T7/Code/zectrix-note-4 && pnpm vitest run src/features/plugins/plugin-market-page.test.tsx`
Expected: 测试通过

- [ ] **Step 7: Commit**

```bash
cd /Volumes/T7/Code/zectrix-note-4
git add src/features/plugins/plugin-market-page.tsx
git commit -m "feat: add category ButtonGroup and filter for builtin plugins"
```

---

### Task 5: 更新 plugin-generator skill

**Files:**
- Modify: `.claude/skills/plugin-generator/SKILL.md`

- [ ] **Step 1: 在输入信息部分新增分类字段**

在 `## 输入信息` 部分，在 `4. **返回格式**` 之后添加：

```markdown
5. **分类** - 例如："AI"、"图片"、"工具"、"文学"、"编程"
```

- [ ] **Step 2: 在生成步骤中新增确定分类步骤**

在 `## 生成步骤` 部分，在 `1. **生成插件 ID**` 和 `2. **生成插件描述**` 之间插入新步骤：

```markdown
2. **确定插件分类** - 接受用户指定的分类，或根据插件类型自动推断：
   - 含"生图/AI/ComfyUI"等关键词 → `"AI"`
   - 含"随机/图片/显示"等关键词 → `"图片"`
   - 含"转/生成/工具"等关键词 → `"工具"`
   - 含"诗词/文学/古文"等关键词 → `"文学"`
   - 含"监控/GitHub/代码"等关键词 → `"编程"`
```

注意：原来的步骤编号需要顺延（原 2→3, 3→4, 4→5）。

- [ ] **Step 3: 更新 builtin_plugins.rs 修改格式新增 category 字段**

在 `## builtin_plugins.rs 修改格式` 部分的代码模板中新增 `category` 字段：

```markdown
```rust
BuiltinPlugin {
    id: "<PLUGIN_ID>".to_string(),
    name: "<插件名称>".to_string(),
    description: "<插件描述>".to_string(),
    category: "<分类>".to_string(),  // 新增
    code: r#"..."# .to_string(),
    ..Default::default()
}
```
```

- [ ] **Step 4: 在示例中新增分类说明**

在 `## 示例` 部分，在输入信息中添加分类：

```markdown
**用户输入：**
- 插件名称：随机显示狗狗
- 接口地址：https://dog.ceo/api/breeds/image/random
- 输出类型：image
- 返回格式：{"message": "图片URL", "status": "success"}
- 分类：图片
```

- [ ] **Step 5: Commit**

```bash
cd /Volumes/T7/Code/zectrix-note-4
git add .claude/skills/plugin-generator/SKILL.md
git commit -m "feat: add category support to plugin-generator skill"
```

---

## Self-Review Checklist

**1. Spec coverage:**
- [x] 数据模型变更（Rust + TypeScript）→ Task 1, Task 3
- [x] 为每个插件设置分类 → Task 2
- [x] ButtonGroup UI 实现 → Task 4 Step 4
- [x] 分类提取与过滤逻辑 → Task 4 Step 2, 3
- [x] plugin-generator skill 修改 → Task 5

**2. Placeholder scan:**
- 无 TBD、TODO、占位符
- 所有步骤包含实际代码

**3. Type consistency:**
- Rust 侧 `category: String` + serde 标注 `skip_serializing_if = "String::is_empty"`
- TypeScript 侧 `category?: string`（可选，与 serde 的 default 行为一致）
- 前端使用 `p.category` 和 `selectedCategory` 进行比较

---

Plan complete and saved to `docs/superpowers/plans/2026-04-27-plugin-category.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
