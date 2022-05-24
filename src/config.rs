use directories::ProjectDirs;
use eyre::{Error, Result};
use serde::{Deserialize, Serialize};
use std;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bridge: Bridge,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bridge {
    pub ip: Option<IpAddr>,
    pub username: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bridge: Bridge {
                ip: None,
                username: None,
            },
        }
    }
}

pub fn get_path() -> Result<PathBuf, Error> {
    let project_dirs = ProjectDirs::from("", "", "blilys").expect("Config dir not readable");
    let config_dir = project_dirs.config_dir();
    if !config_dir.is_dir() {
        fs::create_dir_all(config_dir)?;
    }
    let config_path = config_dir.join("config.toml");
    Ok(config_path)
}

pub fn read_config(config_path: &Path) -> Result<Config, Error> {
    let config: Config = match config_path.is_file() {
        true => toml::from_str(String::from_utf8(fs::read(config_path)?)?.as_ref())?,
        false => Default::default(),
    };
    Ok(config)
}

pub fn print_config(config_path: &Path, config: &Config) -> Result<()> {
    eprintln!("# {}", config_path.to_string_lossy());
    print!("{}", toml::to_string(&config)?);
    Ok(())
}
