use std::{fs, io::Write, path::PathBuf};

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::clash;

pub static CONFIG: Lazy<PathBuf> = Lazy::new(|| {
    let config_dir = dirs_next::config_dir().unwrap_or(PathBuf::from(format!(
        "{}/.config",
        std::env::var("HOME").unwrap_or("".to_string())
    )));

    if !config_dir.exists() {
        let _ = std::fs::create_dir_all(&config_dir);
    }

    config_dir.join("breaddog.conf")
});

#[derive(Debug, Serialize, Deserialize)]
pub struct BreadDogConfig {
    pub url: String,
    pub selector: String,
}

impl BreadDogConfig {
    pub fn new_from_dialoguer(client: &Agent) -> Result<Self> {
        let theme = ColorfulTheme::default();

        let url = Input::<String>::with_theme(&theme)
            .with_prompt("Clash url")
            .default("http://localhost:9090".to_string())
            .interact()?;

        let all_selector = clash::dialoguer_get_selector(client, &url)?;

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
        let v = serde_json::to_vec(&self)?;

        let mut f = fs::File::create(&*CONFIG)?;
        f.write_all(&v)?;

        Ok(())
    }

    pub fn read_from_config() -> Result<Self> {
        let f = fs::read(&*CONFIG)?;
        let result = serde_json::from_slice(&f)?;

        Ok(result)
    }
}
