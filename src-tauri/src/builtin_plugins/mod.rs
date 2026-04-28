use serde::Serialize;

mod bilibili;
mod cat;
mod comfyui;
mod dog;
mod duck;
mod github_actions;
mod github_trending;
mod poetry;
mod qrcode;
mod waifu;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfigOptionItem {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfigOption {
    pub name: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<PluginConfigOptionItem>,
    pub default: String,
}

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
    pub category: String,
}

impl Default for BuiltinPlugin {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            code: String::new(),
            config: vec![],
            supports_loop: true,
            category: String::new(),
        }
    }
}

#[allow(dead_code)]
fn default_true() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

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
