pub mod loader;
pub mod settings;

pub use settings::YshConfig;

use anyhow::Result;

pub fn load_config(path: &str) -> Result<YshConfig> {
    let loader = loader::ConfigLoader::new();
    loader.load_file(path)
}
