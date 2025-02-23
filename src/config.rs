use jsonc_parser::parse_to_serde_value;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};
use tokio::sync::OnceCell;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub author: String,
    pub py_path: String,
    pub s4_mods_path: String,
    pub s4_install_path: String,
}

const SETTINGS_PATH: &str = ".s4m.jsonc";

impl Config {
    pub fn exists() -> bool {
        #[allow(deprecated)]
        let db_path = std::env::home_dir().unwrap().join(SETTINGS_PATH);
        let db_path = db_path.to_str().unwrap();

        Path::new(db_path).exists()
    }

    async fn init(
        author: &str,
        py_path: &str,
        s4_mods_path: &str,
        s4_install_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        #[allow(deprecated)]
        let db_path = std::env::home_dir().unwrap().join(SETTINGS_PATH);
        let db_path = db_path.to_str().unwrap();

        let mut py_path = py_path.replace("/", "\\").replace("\\", "\\\\");
        let mut s4_mods_path = s4_mods_path.replace("/", "\\").replace("\\", "\\\\");
        let mut s4_install_path = s4_install_path.replace("/", "\\").replace("\\", "\\\\");

        if py_path.ends_with("\\") {
            py_path.pop();
        }

        if s4_mods_path.ends_with("\\") {
            s4_mods_path.pop();
        }

        if s4_install_path.ends_with("\\") {
            s4_install_path.pop();
        }

        let contents = format!(
            r#"{{ 
  // The author of the mods
  "author": "{author}",

  // The path to the Python executable
  "py_path": "{py_path}",

  // The path to the Sims 4 mods directory
  "s4_mods_path": "{s4_mods_path}",

  // The path to the Sims 4 installation
  "s4_install_path": "{s4_install_path}",
}}"#
        );

        println!("Writing config to: {}", db_path);
        tokio::fs::write(db_path, &contents).await.unwrap();

        let json_value = parse_to_serde_value(&contents, &Default::default()).unwrap();
        let value: Config = serde_json::from_value(json_value.unwrap()).unwrap();

        Ok(value)
    }

    async fn read() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        #[allow(deprecated)]
        let db_path = std::env::home_dir().unwrap().join(SETTINGS_PATH);
        let db_path = db_path.to_str().unwrap();

        let data = tokio::fs::read_to_string(db_path).await?;
        let json_value = match parse_to_serde_value(&data, &Default::default())? {
            Some(json_value) => json_value,
            None => {
                return Err("Failed to parse JSON value".into());
            }
        };

        let value: Config = serde_json::from_value(json_value)?;
        Ok(value)
    }
}

static CONFIG_INSTANCE: OnceCell<Arc<Config>> = OnceCell::const_new();

pub async fn write_config(
    author: &str,
    py_path: &str,
    s4_mods_path: &str,
    s4_install_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::init(author, py_path, s4_mods_path, s4_install_path).await?;
    if let Err(e) = CONFIG_INSTANCE.set(Arc::new(config)) {
        return Err(e.into());
    };

    Ok(())
}

pub async fn load_config() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::read().await?;
    if let Err(e) = CONFIG_INSTANCE.set(Arc::new(config)) {
        return Err(e.into());
    };

    Ok(())
}

pub async fn get_config() -> Arc<Config> {
    CONFIG_INSTANCE.get().unwrap().clone()
}
