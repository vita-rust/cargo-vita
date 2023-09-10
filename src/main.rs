mod check;
mod commands;
mod meta;
mod nc;

use check::check_rust_version;
use clap::Parser;
use commands::{Cargo, Executor};

fn main() {
    check_rust_version();

    let Cargo::Input(input) = Cargo::parse();
    input.cmd.execute(input.verbose);
}
