use clash::get_proxy_dialoguer;
use config::{config_is_exist, BreadDogConfig};
use reqwest::blocking::Client;

mod clash;
mod config;

fn main() {
    let config_is_exist = config_is_exist();

    let client = Client::new();

    let config = if !config_is_exist {
        BreadDogConfig::new_from_dialoguer(&client)
            .expect("Unknown Error") 
    } else {
        BreadDogConfig::read_from_config().expect("Can not get config file")
    };

    if let Err(e) = get_proxy_dialoguer(&client, config) {
        eprintln!("{}", e);
    }
}
