mod check;
mod commands;
mod meta;
mod nc;

use check::check_rust_version;
use clap::Parser;
use colored::Colorize;
use commands::{Cargo, Executor};

fn main() {
    check_rust_version();

    let Cargo::Input(input) = Cargo::parse();
    match input.cmd.execute(input.verbose) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {}", "Error:".bold().red(), format!("{e:?}").red());
        }
    }
}
