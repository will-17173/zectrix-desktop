# 内置插件分类设计

## 概述

为内置插件列表新增子分类功能，使用按钮组（ButtonGroup）切换分类，默认显示"全部"。同时更新 plugin-generator skill，在创建插件时收集分类信息。

## 分类映射

| 插件 | 分类 |
|------|------|
| ComfyUI 生图 | AI |
| 随机显示猫猫/狗狗/鸭子/Waifu | 图片 |
| 文本转二维码 | 工具 |
| 随机古诗词 | 文学 |
| GitHub Actions 监控 | 编程 |

## 1. 数据模型变更

### Rust 侧 (`src-tauri/src/builtin_plugins.rs`)

`BuiltinPlugin` struct 新增 `category` 字段：

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

为每个插件设置分类：

```rust
pub fn list_builtin_plugins() -> Vec<BuiltinPlugin> {
    vec![
        BuiltinPlugin {
            id: "comfyui-image".to_string(),
            name: "ComfyUI 生图".to_string(),
            description: "调用 ComfyUI 生成图片并推送到设备".to_string(),
            category: "AI".to_string(),  // 新增
            // ...
        },
        BuiltinPlugin {
            id: "cat-random".to_string(),
            // ...
            category: "图片".to_string(),  // 新增
        },
        // ... 其他插件类似
    ]
}
```

### TypeScript 侧 (`src/lib/tauri.ts`)

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

## 2. ButtonGroup UI 实现

### 位置

在"内置插件"TabContent 内的**最上方**，在现有提示文字之前。

### 组件结构

使用一组按钮模拟胶囊状分类选择器，不使用 shadcn/ui 的 Tabs：

```tsx
const [selectedCategory, setSelectedCategory] = useState("");

const categories = useMemo(() => {
  const cats = new Set<string>();
  builtinPlugins.forEach(p => { if (p.category) cats.add(p.category); });
  return [
    { key: "", label: "全部" },
    ...Array.from(cats).sort().map(c => ({ key: c, label: c }))
  ];
}, [builtinPlugins]);
```

```tsx
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
```

### 样式说明

- 外层容器：`flex items-center rounded-md border border-gray-300 overflow-hidden`
- 选中态：`bg-blue-500 text-white shadow-sm`
- 未选中态：`bg-white text-gray-600 hover:bg-gray-50`
- 边框处理：除最后一个按钮外都有 `border-r border-gray-300`，首尾按钮处理圆角

## 3. 分类提取与过滤逻辑

### 提取分类列表

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

### 过滤插件列表

```tsx
const filteredPlugins = useMemo(() => {
  if (!selectedCategory) return builtinPlugins;
  return builtinPlugins.filter(p => p.category === selectedCategory);
}, [builtinPlugins, selectedCategory]);
```

### 组件结构

```
TabsContent "builtin"
  ├── ButtonGroup (分类选择，mb-3)
  ├── p (提示文字)
  └── div.grid (插件列表，使用 filteredPlugins)
        └── PluginCard × filteredPlugins.length
```

## 4. plugin-generator skill 修改

更新 `/Volumes/T7/Code/zectrix-note-4/.claude/skills/plugin-generator/SKILL.md`：

### 输入信息部分新增

- **分类** - 例如："AI"、"图片"、"工具"、"文学"、"编程"

### 生成步骤新增

在生成插件 ID 之后、生成插件描述之前，新增：

- **确定插件分类** - 接受用户指定的分类，或根据插件类型自动推断：
  - 含"生图/AI/ComfyUI"等关键词 → `"AI"`
  - 含"随机/图片/显示"等关键词 → `"图片"`
  - 含"转/生成/工具"等关键词 → `"工具"`
  - 含"诗词/文学/古文"等关键词 → `"文学"`
  - 含"监控/GitHub/代码"等关键词 → `"编程"`

### builtin_plugins.rs 修改格式新增 `category` 字段

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

## 需要改动的文件

1. `src-tauri/src/builtin_plugins.rs` — 新增 `category` 字段 + 为每个插件设置分类
2. `src/lib/tauri.ts` — TypeScript 类型新增 `category?`
3. `src/features/plugins/plugin-market-page.tsx` — 新增 ButtonGroup + 过滤逻辑
4. `.claude/skills/plugin-generator/SKILL.md` — 更新 skill 以包含分类信息
