
use std::path::PathBuf;
use std::path::Path;

use std::env::home_dir;
use std::env;

pub fn get_config_path() -> Result<PathBuf, String> {
    let mut config_path = PathBuf::new();
    match env::var("KAKAPO_HOME") {
        Ok(kakapo_home) => {
            config_path.push(kakapo_home);
        },
        Err(_) => {
            let home = home_dir()
                .ok_or_else(|| "No home directory found, please specify an explicit 'KAKAPO_HOME' environment".to_string())?;
            config_path.push(home);
            config_path.push(".kakapo");
        },
    }

    config_path.push("config.yaml");
    Ok(config_path)
}