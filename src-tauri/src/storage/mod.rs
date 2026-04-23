use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::Path};

pub fn load_json<T>(path: &Path) -> anyhow::Result<T>
where
    T: DeserializeOwned + Default,
{
    if !path.exists() {
        return Ok(T::default());
    }
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

pub fn save_json<T>(path: &Path, value: &T) -> anyhow::Result<()>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
