use std::{process::exit, rc::Rc};

use anyhow::Result;
use clap::{Parser, Subcommand};
use clash::{get_all_speed, get_proxy_dialoguer};
use config::{BreadDogConfig, CONFIG};
use dialoguer::console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::{colors::xterm::Gray, OwoColorize};
use ureq::Agent;

mod clash;
mod config;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    subcmd: Option<Subcmd>,
}

#[derive(Subcommand, Debug)]
enum Subcmd {
    /// Select a proxy
    Menu,
    /// Speedtest all proxy
    Speedtest,
}

fn main() {
    env_logger::init();
    ctrlc::set_handler(ctrlc_handler).expect("Can not set ctrlc handler");
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        exit(1);
    }

    exit(0);
}

fn try_main() -> Result<()> {
    let agent = ureq::AgentBuilder::new().build();

    let config = if !CONFIG.is_file() {
        BreadDogConfig::new_from_dialoguer(&agent)?
    } else {
        BreadDogConfig::read_from_config()?
    };

    let args = Args::parse();
    match args.subcmd {
        Some(Subcmd::Menu) | None => get_proxy_dialoguer(&agent, config)?,
        Some(Subcmd::Speedtest) => speedtest(&agent, &config)?,
    }

    Ok(())
}

fn speedtest(agent: &Agent, config: &BreadDogConfig) -> Result<()> {
    let pb = Rc::new(ProgressBar::new(0));
    pb.set_style(
        ProgressStyle::with_template("[{wide_bar:.cyan/blue}] ({pos}/{len})")
            .unwrap()
            .progress_chars("=>-"),
    );
    let pbc = pb.clone();
    let pbcc = pb.clone();
    get_all_speed(
        agent,
        config,
        move |proxy, delay, mean_delay, len| {
            if pb.length().map(|x| x == 0).unwrap_or(true) {
                pb.set_length(len as u64);
            }
            let number_str = |x| match x {
                0..=300 => x.green().to_string(),
                301..=800 => x.yellow().to_string(),
                801.. => x.red().to_string(),
            };

            let delay_str = number_str(delay);
            let other_str = |x: &str| match delay {
                0..=300 => x.green().to_string(),
                301..=800 => x.yellow().to_string(),
                801.. => x.red().to_string(),
            };

            pb.println(format!(
                "{} {delay_str}{}",
                other_str(&format!("{proxy}:")),
                match mean_delay {
                    Some(mean_delay) => other_str(&format!("(mean delay: {mean_delay})")),
                    None => "".to_string(),
                }
            ));
            pb.inc(1);
        },
        move |err, len| {
            if pbc.length().map(|x| x == 0).unwrap_or(true) {
                pbc.set_length(len as u64);
            }
            pbc.println(format!("{}", err.fg::<Gray>()));
            pbc.inc(1);
        },
    )?;

    pbcc.finish_and_clear();

    Ok(())
}

fn ctrlc_handler() {
    let _ = Term::stdout().show_cursor();
    exit(1);
}
