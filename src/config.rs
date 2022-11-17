use std::{fs, io::{Write, Read}, path::PathBuf};

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::clash;

#[derive(Debug, Serialize, Deserialize)]
pub struct BreadDogConfig {
    pub url: String,
    pub selector: String,
}

impl BreadDogConfig {
    pub async fn new_from_dialoguer(client: &Client) -> Result<Self> {
        let theme = ColorfulTheme::default();

        let url = Input::<String>::with_theme(&theme)
            .with_prompt("Clash url")
            .default("http://localhost:9090".to_string())
            .interact()?;

        let all_selector = clash::dialoguer_get_selector(client, &url).await?;

        let all_selector = all_selector.keys().collect::<Vec<_>>();

        let selector = Select::with_theme(&ColorfulTheme::default())
            .default(0)
            .with_prompt("Choose a selector to switch proxy")
            .items(&all_selector)
            .interact()?;

        let result = Self {
            url,
            selector: all_selector[selector].to_string(),
        };

        result.save_config()?;

        Ok(result)
    }

    fn save_config(&self) -> Result<()> {
        let s = serde_json::to_string(&self)?;

        let mut f = fs::File::create(config_path())?;
        f.write_all(s.as_bytes())?;

        Ok(())
    }

    pub fn read_from_config() -> Result<Self> {
        let config_path = config_path();

        let mut buf = Vec::new();

        let mut f = fs::File::open(config_path)?;
        f.read_to_end(&mut buf)?;

        let result = serde_json::from_slice(&buf)?;

        Ok(result)
    }
}

pub fn config_is_exist() -> bool {
    let config_path = config_path();

    if config_path.is_file() {
        return true;
    }

    return false;
}

fn config_path() -> PathBuf {
    let config_dir = dirs_next::config_dir().expect("Can not get system config dir path!");
    let config_path = config_dir.join("breaddog.conf");

    config_path
}
