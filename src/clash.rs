use std::collections::HashMap;

use anyhow::{anyhow, bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use serde::Deserialize;
use ureq::{Agent, Error};

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

#[derive(Deserialize, Debug)]
pub struct ClashDelay {
    delay: u64,
    #[serde(rename = "meanDelay")]
    mean_delay: u64,
}

fn get_all(client: &Agent, url: &str) -> Result<ClashResult> {
    let resp = client
        .get(url)
        .call()
        .context("Can not get clash resful API")?;

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
    let json = get_single_selector(client, &config)?;

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

fn get_single_selector(client: &Agent, config: &BreadDogConfig) -> Result<ClashProxies> {
    let resp = client
        .get(&format!("{}/proxies/{}", config.url, config.selector))
        .call()
        .context("Can not get clash resful API")?;
    let json = resp.into_json::<ClashProxies>()?;

    Ok(json)
}

pub fn get_all_speed<F, F2>(
    client: &Agent,
    config: &BreadDogConfig,
    mut callback: F,
    mut err_callback: F2,
) -> Result<()>
where
    F: FnMut(String, u64, u64, usize),
    F2: FnMut(&str, usize),
{
    let selector = get_single_selector(client, config)?;
    let all = selector.all.context("no all")?;

    let len = all.len();

    for i in all {
        let speed = get_single_speed(client, config, &i);
        match speed {
            Ok((delay, mean_delay)) => callback(i, delay, mean_delay, len),
            Err(e) => err_callback(&e.to_string(), len),
        }
    }

    Ok(())
}

fn get_single_speed(client: &Agent, config: &BreadDogConfig, proxy: &str) -> Result<(u64, u64)> {
    let res = match client
        .get(&format!("{}/proxies/{proxy}/delay", config.url))
        .query("url", "https://google.com")
        .query("timeout", "10000")
        .call()
    {
        Ok(res) => res,
        Err(e) => match e {
            Error::Status(503, _) => bail!("{proxy} proxy unavailable"),
            Error::Status(504, _) => bail!("{proxy} timeout"),
            e => return Err(e.into()),
        },
    };

    let delay = res.into_json::<ClashDelay>()?;
    let res = (delay.delay, delay.mean_delay);

    Ok(res)
}
