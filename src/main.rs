use anyhow::Result;
use clash::get_proxy_dialoguer;
use config::{BreadDogConfig, CONFIG};
use dialoguer::console::Term;

mod clash;
mod config;

fn main() {
    ctrlc::set_handler(|| ctrlc_handler()).expect("Can not set ctrlc handler");
    if let Err(e) = try_main() {
        eprintln!("{}", e);
    }
}

fn try_main() -> Result<()> {
    let agent = ureq::AgentBuilder::new().build();

    let config = if !CONFIG.is_file() {
        BreadDogConfig::new_from_dialoguer(&agent)?
    } else {
        BreadDogConfig::read_from_config()?
    };

    get_proxy_dialoguer(&agent, config)?;

    Ok(())
}

fn ctrlc_handler() {
    let _ = Term::stdout().show_cursor();
}