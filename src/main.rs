use std::process::exit;

use anyhow::Result;
use clash::{get_all_speed, get_proxy_dialoguer};
use config::{BreadDogConfig, CONFIG};
use dialoguer::console::Term;
use owo_colors::{colors::xterm::Gray, OwoColorize};

mod clash;
mod config;

fn main() {
    ctrlc::set_handler(ctrlc_handler).expect("Can not set ctrlc handler");
    if let Err(e) = try_main() {
        eprintln!("{}", e);
    }
}

fn try_main() -> Result<()> {
    let args = std::env::args();

    let agent = ureq::AgentBuilder::new().build();

    let config = if !CONFIG.is_file() {
        BreadDogConfig::new_from_dialoguer(&agent)?
    } else {
        BreadDogConfig::read_from_config()?
    };

    match args.last().as_deref() {
        Some("speedtest") => get_all_speed(
            &agent,
            &config,
            |proxy, delay, mean_delay| {
                let number_str = |x| match x {
                    0..=300 => x.green().to_string(),
                    301..=800 => x.yellow().to_string(),
                    801.. => x.red().to_string(),
                };

                let delay_str = number_str(delay);
                let mean_delay_str = number_str(mean_delay);

                let other_str = |x: &str| match mean_delay {
                    0..=300 => x.green().to_string(),
                    301..=800 => x.yellow().to_string(),
                    801.. => x.red().to_string(),
                };

                println!(
                    "{} {delay_str} {} {mean_delay_str}{}",
                    other_str(&format!("{proxy}:")),
                    other_str("(mean:"),
                    other_str(")")
                );
            },
            |err| {
                eprintln!("{}", err.fg::<Gray>());
            },
        )?,
        Some(_) => get_proxy_dialoguer(&agent, config)?,
        None => unreachable!(),
    }

    Ok(())
}

fn ctrlc_handler() {
    let _ = Term::stdout().show_cursor();
    exit(1);
}
