---
name: plugin-generator
description: 生成新的内置插件。当用户说"创建插件"、"新增插件"、"生成插件"并提供API接口地址和插件名称时触发。用户会说明输出类型(image/text)和API返回格式。
---

# 插件生成器

根据用户提供的 API 接口地址、插件名称、输出类型和返回格式，生成新的内置插件代码并自动注册到 `src-tauri/src/builtin_plugins.rs`。

## 输入信息

用户会提供以下信息：
1. **插件名称** - 例如："随机显示狗狗"、"今日天气"
2. **接口地址** - 例如："https://dog.ceo/api/breeds/image/random"
3. **输出类型** - `image` 或 `text`
4. **返回格式** - 描述 API 返回的 JSON 结构，例如：`{"message": "图片URL", "status": "success"}`
5. **分类** - 例如："AI"、"图片"、"工具"、"文学"、"编程"（可选，不提供时根据插件类型自动推断）

## 插件模板

### 图片类型插件模板

图片插件需要：
1. 调用 API 获取图片 URL
2. 使用 `fetchBase64` 下载图片转成 base64
3. 返回 `{ type: "image", imageDataUrl: "...", title: "..." }`

```javascript
const data = await fetchJson("<API_URL>");
const imageDataUrl = await fetchBase64(data.<IMAGE_URL_FIELD>);
return { type: "image", imageDataUrl: imageDataUrl, title: "<PLUGIN_NAME>" };
```

### 文本类型插件模板

文本插件需要：
1. 调用 API 获取文本数据
2. 返回 `{ type: "text", text: "...", title: "..." }`

```javascript
const data = await fetchJson("<API_URL>");
return { type: "text", text: data.<TEXT_FIELD>, title: "<PLUGIN_NAME>" };
```

### textImage 类型插件模板

如果需要渲染文本为图片再推送：
```javascript
const data = await fetchJson("<API_URL>");
return { type: "textImage", text: data.<TEXT_FIELD>, title: "<PLUGIN_NAME>" };
```

## 生成步骤

1. **生成插件 ID** - 根据插件名称生成英文 ID
   - 提取关键词翻译成英文，例如："随机显示狗狗" → `dog-random`
   - "今日天气" → `weather-today`
   - 使用小写字母和连字符

2. **确定插件分类** - 接受用户指定的分类，或根据插件类型自动推断：
   - 含"生图/AI/ComfyUI"等关键词 → `"AI"`
   - 含"随机/图片/显示"等关键词 → `"图片"`
   - 含"转/生成/工具"等关键词 → `"工具"`
   - 含"诗词/文学/古文"等关键词 → `"文学"`
   - 含"监控/GitHub/代码"等关键词 → `"编程"`

3. **生成插件描述** - 根据插件功能生成简短描述
   - 例如："随机获取一张狗狗图片并推送到设备"

4. **生成插件代码** - 根据输出类型和返回格式生成 JS 代码
   - 图片类型：使用 `fetchJson` + `fetchBase64`
   - 文本类型：使用 `fetchJson`
   - 根据用户提供的返回格式提取正确的字段

5. **更新 builtin_plugins.rs** - 在 `list_builtin_plugins()` 函数中添加新的 `BuiltinPlugin`

## builtin_plugins.rs 修改格式

在 `src-tauri/src/builtin_plugins.rs` 的 `list_builtin_plugins()` 函数的 `vec!` 中添加：

```rust
BuiltinPlugin {
    id: "<PLUGIN_ID>".to_string(),
    name: "<插件名称>".to_string(),
    description: "<插件描述>".to_string(),
    category: "<分类>".to_string(),
    code: r#"const data = await fetchJson("<API_URL>");
const imageDataUrl = await fetchBase64(data.<IMAGE_URL_FIELD>);
return { type: "image", imageDataUrl: imageDataUrl, title: "<插件名称>" };"#
        .to_string(),
    ..Default::default()
},
```

## 执行流程

1. 读取当前的 `src-tauri/src/builtin_plugins.rs` 文件
2. 根据用户输入生成插件 ID、描述和代码
3. 使用 Edit 工具在 `list_builtin_plugins()` 的 `vec!` 中添加新的插件项
4. 运行 `cargo build --manifest-path src-tauri/Cargo.toml` 验证编译
5. 告知用户插件已生成并可使用

## 示例

**用户输入：**
- 插件名称：随机显示狗狗
- 接口地址：https://dog.ceo/api/breeds/image/random
- 输出类型：image
- 返回格式：{"message": "图片URL", "status": "success"}
- 分类：图片

**生成结果：**
- 插件 ID：`dog-random`
- 插件代码：
```javascript
const data = await fetchJson("https://dog.ceo/api/breeds/image/random");
const imageDataUrl = await fetchBase64(data.message);
return { type: "image", imageDataUrl: imageDataUrl, title: "随机显示狗狗" };
```