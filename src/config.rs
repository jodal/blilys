use directories::ProjectDirs;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing)]
    pub path: Option<PathBuf>,

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
            path: None,
            bridge: Bridge {
                ip: None,
                username: None,
            },
        }
    }
}

impl Config {
    pub fn from_file() -> Result<Config> {
        Ok(Config::read_file(&Config::get_path()?)?)
    }

    fn get_path() -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("", "", "blilys").expect("Config dir not readable");
        let config_dir = project_dirs.config_dir();
        if !config_dir.is_dir() {
            fs::create_dir_all(config_dir)?;
        }
        let config_path = config_dir.join("config.toml");
        Ok(config_path)
    }

    fn read_file(path: &Path) -> Result<Config> {
        let mut config: Config = match path.is_file() {
            true => toml::from_str(String::from_utf8(fs::read(path)?)?.as_ref())?,
            false => Default::default(),
        };
        config.path = Some(path.to_owned());
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = self
            .path
            .as_ref()
            .expect("Config must have a path to be saved.");
        let contents = toml::to_string(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn print(&self) -> Result<()> {
        if let Some(path) = &self.path {
            eprintln!("# {}", path.display());
        }
        print!("{}", toml::to_string(self)?);
        Ok(())
    }
}
