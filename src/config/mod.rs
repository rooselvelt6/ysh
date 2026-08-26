pub mod lua_engine;
pub mod settings;

pub use settings::YshConfig;

use anyhow::Result;

pub fn load_config(path: &str) -> Result<YshConfig> {
    let lua = lua_engine::LuaEngine::new()?;
    lua.load_file(path)
}
