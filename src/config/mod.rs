use std::{
    env::home_dir,
    fs::{create_dir_all, read},
    path::PathBuf,
    str::FromStr,
};

use crate::config::webclient_config::WebClientConfig;
use anyhow::{Context, Result, anyhow};
use reqwest::Url;
use serde::{Deserialize, Serialize};

pub mod webclient_config;

const DEFAULT_CONFIG_FOLDER: &str = ".config/rust-browser";
const APP_CONFIG_FILE: &str = "app.toml";

#[derive(Debug, Deserialize, Serialize)]
pub struct Configs {
    #[serde(skip)]
    config_file_name: &'static str,
    #[serde(skip)]
    config_path: &'static str,
    pub webclient_config: WebClientConfig,
}

impl Default for Configs {
    fn default() -> Self {
        Self {
            config_file_name: APP_CONFIG_FILE,
            config_path: DEFAULT_CONFIG_FOLDER,
            webclient_config: Default::default(),
        }
    }
}

// https://www.reddit.com/r/rust/comments/11r0ux3/comment/jc73o63/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button
impl Configs {
    pub fn new() -> Result<Self> {
        let d = Self::default();
        match d.read_config() {
            Ok(data) => Ok(data),
            Err(e) => Err(e),
        }
    }

    /// This method tries to check if the config file exists, returning its path
    fn try_get_config(&self) -> Result<PathBuf> {
        let home = home_dir().context("Cannot find home dir!")?;

        // this represents the config file path
        let path = home.join(self.config_path);

        if !path.exists() {
            create_dir_all(&path).context("Failed to create dir!")?;
        }

        Ok(path)
    }

    fn get_config_file(&self) -> Result<PathBuf> {
        Ok(self.try_get_config()?.join(self.config_file_name))
    }

    /// method for creating config files
    pub fn create_config(&self) -> Result<()> {
        let content =
            toml::to_string_pretty(self).context("TOML crate failed trying to create_config!")?;

        let path = self.try_get_config()?;

        // this represents the path + file name
        let complete_filepath = path.join(self.config_file_name);

        std::fs::write(&complete_filepath, content)
            .context(format!("Cannot write to {:#?}", complete_filepath))?;

        Ok(())
    }

    pub fn read_config(&self) -> Result<Configs> {
        let path = self.get_config_file()?;

        if !path.exists() {
            return Err(anyhow!("{:?} does not exist", path));
        }

        let file = read(&path);

        if !file.is_ok() {
            self.create_config()?;
        }

        let text = String::from_utf8(file?)?;

        let config: Configs = toml::from_str(&text)?;

        // an invalid URL should crash the app
        match Url::from_str(&config.webclient_config.search_url) {
            Ok(_) => {}
            Err(err) => {
                return Err(anyhow!(
                    "{} is not a valid url!\nParse Error: {}",
                    config.webclient_config.search_url,
                    err.to_string()
                ));
            }
        };


        Ok(Configs {
            config_file_name: self.config_file_name,
            config_path: self.config_path,
            ..config
        })
    }
}
