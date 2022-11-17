use std::collections::HashMap;

use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use reqwest::Client;
use serde::Deserialize;

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

async fn get_all(client: &Client, url: &str) -> Result<ClashResult> {
    let resp = client.get(url).send().await?.error_for_status()?;

    let json = resp.json::<ClashResult>().await?;

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

pub async fn dialoguer_get_selector(
    client: &Client,
    url: &str,
) -> Result<HashMap<String, ClashProxies>> {
    let all = get_all(client, &format!("{}/proxies", url)).await?;

    let selector = get_all_selector(all)?;

    Ok(selector)
}

pub async fn get_proxy_dialoguer(client: &Client, config: BreadDogConfig) -> Result<()> {
    let resp = client
        .get(format!("{}/proxies/{}", config.url, config.selector))
        .send()
        .await?;
    let json = resp.json::<ClashProxies>().await?;

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
        return Ok(())
    }

    let mut json = HashMap::new();
    json.insert("name", all[select_index].clone());

    client
        .put(format!("{}/proxies/{}", config.url, config.selector))
        .json(&json)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[tokio::test]
async fn test() {
    let all = get_all(&Client::new(), "http://localhost:9092/proxies")
        .await
        .unwrap();

    let selector = get_all_selector(all).unwrap();

    dbg!(selector);
}
