use anyhow::{Context, Result};
use mlua::Lua;
use std::path::Path;

use super::settings::YshConfig;

#[derive(Clone)]
pub struct LuaEngine {
    lua: Lua,
}

impl LuaEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self { lua })
    }

    pub fn load_file(&self, path: &str) -> Result<YshConfig> {
        let path = Path::new(path);
        let code = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let result: mlua::Table = self
            .lua
            .load(&code)
            .eval()
            .map_err(|e| anyhow::anyhow!("Failed to evaluate Lua config: {}: {}", path.display(), e))?;

        YshConfig::from_lua_table(result)
    }

    pub fn reload(&self, path: &str) -> Result<YshConfig> {
        tracing::info!("Reloading Lua config from: {}", path);
        self.load_file(path)
    }
}
