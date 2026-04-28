# 设计：插件模块化 + 股票多市场支持

日期：2026-04-28

## 一、内置插件模块化

### 问题

`src-tauri/src/builtin_plugins.rs` 是单个 597 行文件，所有插件定义堆在一起。每次新增或修改插件都要编辑同一个文件，随插件增多维护成本增大。

### 目标结构

```
src-tauri/src/builtin_plugins/
├── mod.rs             # 类型定义 + list_builtin_plugins() + find_builtin_plugin()
├── comfyui.rs
├── cat.rs
├── dog.rs
├── duck.rs
├── waifu.rs
├── qrcode.rs
├── poetry.rs
├── github_actions.rs
├── bilibili.rs
└── github_trending.rs
```

原 `builtin_plugins.rs` 文件删除，替换为同名目录模块。

### 每个插件文件结构

```rust
use crate::builtin_plugins::BuiltinPlugin;

pub fn plugin() -> BuiltinPlugin {
    BuiltinPlugin {
        id: "cat-random".to_string(),
        name: "随机显示猫猫".to_string(),
        // ...
    }
}
```

### mod.rs 结构

```rust
// 类型定义（PluginConfigOption、PluginConfigOptionItem、BuiltinPlugin）保持不变

mod comfyui;
mod cat;
// ... 其余模块

pub fn list_builtin_plugins() -> Vec<BuiltinPlugin> {
    vec![
        comfyui::plugin(),
        cat::plugin(),
        dog::plugin(),
        duck::plugin(),
        waifu::plugin(),
        qrcode::plugin(),
        poetry::plugin(),
        github_actions::plugin(),
        bilibili::plugin(),
        github_trending::plugin(),
    ]
}

pub fn find_builtin_plugin(id: &str) -> Option<BuiltinPlugin> {
    list_builtin_plugins()
        .into_iter()
        .find(|plugin| plugin.id == id)
}
```

### 对外接口

`lib.rs` 的 `use crate::builtin_plugins::*` 无需修改，外部调用方完全不感知拆分。

---

## 二、股票多市场支持（A 股 / 港股 / 美股）

### 市场识别规则

输入自动识别，无需用户手动选市场：

| 输入示例 | 市场 | 规则 |
|---------|------|------|
| `600519`、`000001` | A 股（`a`） | 6 位纯数字 |
| `700`、`00700`、`9988` | 港股（`hk`） | 1-5 位纯数字，统一左补零到 5 位 |
| `AAPL`、`BRK.B`、`BABA` | 美股（`us`） | 含字母（大小写均可，统一转大写） |

### 数据模型变化

`StockWatchRecord`（`models.rs`）新增 `market` 字段：

```rust
pub struct StockWatchRecord {
    pub code: String,    // 标准化后：A股6位 / 港股5位 / 美股原样大写
    pub market: String,  // "a" | "hk" | "us"
    pub created_at: String,
}
```

向后兼容：现有 `stock_watchlist.json` 无 `market` 字段时，反序列化默认为 `"a"`。

### stock_quote.rs 变化

新增 `StockCode` 结构体替代裸 `String`：

```rust
pub struct StockCode {
    pub code: String,
    pub market: String,
}

pub fn parse_stock_input(input: &str) -> anyhow::Result<StockCode>
```

腾讯行情前缀映射扩展：

```
A 股：sh600519 / sz000001 / bj830000（原有逻辑不变）
港股：hk00700
美股：usAAPL
```

`parse_tencent_quotes` 的字段索引逻辑不变，腾讯格式三市场一致。

`format_stock_push_text` 显示格式：

```
AAPL 苹果公司 189.30 +1.20 +0.64% [美]
00700 腾讯控股 380.00 -2.00 -0.52% [港]
600519 贵州茅台 1458.49 +39.49 +2.78%     ← A 股不加标签
```

### 前端变化

- `StockWatchRecord` TypeScript 类型加 `market: string` 字段
- 添加股票输入框 placeholder：`A股: 600519 | 港股: 00700 | 美股: AAPL`
- 股票列表行加市场标签 badge（A 股不显示，港股显示「港」，美股显示「美」）

### 数据流

```
用户输入 → parse_stock_input() 识别市场
         → add_stock_watch() 保存含 market 的 StockWatchRecord
         → fetch_stock_quotes() 按 market 生成腾讯前缀
         → 同一 HTTP 请求返回混合市场报价
         → format_stock_push_text() 格式化含市场标识
```
