use std::collections::HashMap;

use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use serde::Deserialize;
use ureq::Agent;

use crate::config::BreadDogConfig;

#[derive(Deserialize, Debug)]
struct ClashResult {
    proxies: HashMap<String, ClashProxies>,
}

#[derive(Deserialize, Debug)]
pub struct ClashProxies {
    all: Option<Vec<String>>,
    now: Option<String>,
    #[serde(rename = "type")]
    item_type: String,
}

fn get_all(client: &Agent, url: &str) -> Result<ClashResult> {
    let resp = client
        .get(url)
        .call()
        .map_err(|e| anyhow!("Can not get Clash Resful API, why: {e}"))?;

    let json = resp.into_json::<ClashResult>()?;

    Ok(json)
}

fn get_all_selector(json: ClashResult) -> Result<HashMap<String, ClashProxies>> {
    let proxies = json.proxies;

    let selector = proxies
        .into_iter()
        .filter(|(_, y)| y.item_type == "Selector")
        .collect::<HashMap<String, ClashProxies>>();

    Ok(selector)
}

pub fn dialoguer_get_selector(client: &Agent, url: &str) -> Result<HashMap<String, ClashProxies>> {
    let all = get_all(client, &format!("{}/proxies", url))?;

    let selector = get_all_selector(all)?;

    Ok(selector)
}

pub fn get_proxy_dialoguer(client: &Agent, config: BreadDogConfig) -> Result<()> {
    let resp = client
        .get(&format!("{}/proxies/{}", config.url, config.selector))
        .call()
        .map_err(|e| anyhow!("Can not get clash resful API, why: {e}"))?;

    let json = resp.into_json::<ClashProxies>()?;

    let now = json
        .now
        .ok_or_else(|| anyhow!("Can not get current proxy!"))?;
    let all = json.all.ok_or_else(|| anyhow!("Can not get proxy list"))?;

    let now_index = all
        .iter()
        .position(|x| x == &now)
        .ok_or_else(|| anyhow!("Can not get current proxy in proxy list index"))?;

    let theme = ColorfulTheme::default();
    let select_index = Select::with_theme(&theme)
        .items(&all)
        .default(now_index)
        .with_prompt("Select proxy")
        .interact()?;

    if all[select_index] == now {
        return Ok(());
    }

    let mut json = HashMap::new();
    json.insert("name", all[select_index].as_str());

    client
        .put(&format!("{}/proxies/{}", config.url, config.selector))
        .send_json(json)
        .map_err(|e| anyhow!("Can not switch to proxy {}, why: {e}", all[select_index]))?;

    Ok(())
}
