mod check;
mod commands;
mod ftp;
mod meta;
mod nc;

use clap::Parser;
use colored::Colorize;
use commands::{Cargo, Executor};
use log::error;

fn main() {
    let _ = check::set_cargo_config_env();

    let Cargo::Input(input) = Cargo::parse();

    env_logger::Builder::new()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(match (input.quiet, input.verbose) {
            (true, _) => log::LevelFilter::Error,
            (false, 0) => log::LevelFilter::Info,
            (false, 1) => log::LevelFilter::Debug,
            (false, _) => log::LevelFilter::Trace,
        })
        .init();

    let Cargo::Input(input) = Cargo::parse();
    match input.cmd.execute() {
        Ok(()) => {}
        Err(e) => {
            error!("{}", format!("{e:?}").red());
            std::process::exit(1);
        }
    }
}
