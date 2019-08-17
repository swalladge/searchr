use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tantivy::tokenizer::Language;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub main: MainConfig,
    pub indexes: HashMap<String, IndexConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MainConfig {
    pub default_index: Option<String>,
}

fn default_language() -> Language {
    Language::English
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct IndexConfig {
    pub index_path: String,
    #[serde(default = "default_language")]
    pub language: Language,
    pub files: Vec<String>,
    pub case_sensitive: Option<bool>,
    pub require_literal_separator: Option<bool>,
    pub require_literal_leading_dot: Option<bool>,
}

impl Config {
    pub fn load(file_override: Option<String>) -> Result<Self, Box<dyn Error>> {
        let path: String = match file_override {
            Some(s) => {
                if Path::new(&s).exists() {
                    s.to_owned()
                } else {
                    return Err(format!("config file not found: {:?}", s).into());
                }
            }
            None => {
                // no file override; find a file in the default locations
                let config_dir = match env::var("XDG_CONFIG_HOME") {
                    Ok(val) => val,
                    Err(_) => format!("{}/.config", env::var("HOME")?),
                };

                let config_file = format!("{}/local-search/config.toml", config_dir);

                if Path::new(&config_file).exists() {
                    config_file
                } else {
                    return Err(format!(
                        "no config file found. Create a new one in {}",
                        config_file
                    )
                    .into());
                }
            }
        };

        let mut file = File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let config = toml::from_str(&data)?;
        Ok(config)
    }
}
