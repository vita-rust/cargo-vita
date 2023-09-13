mod check;
mod commands;
mod ftp;
mod meta;
mod nc;

use check::check_rust_version;
use clap::Parser;
use colored::Colorize;
use commands::{Cargo, Executor};

fn main() {
    check_rust_version();
    let _ = check::set_cargo_config_env();

    let Cargo::Input(input) = Cargo::parse();
    match input.cmd.execute(1 + input.verbose - input.quiet as u8) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {}", "Error:".bold().red(), format!("{e:?}").red());
        }
    }
}
