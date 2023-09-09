mod commands;
mod nc;
mod version;

use clap::Parser;
use commands::Cargo;
use version::check_rust_version;

fn main() {
    check_rust_version();

    let Cargo::Input(input) = Cargo::parse();

    match input.cmd {
        commands::CargoCmd::Logs(cmd) => cmd.execute(input.verbose),
        commands::CargoCmd::Run(cmd) => cmd.execute(input.verbose),
        commands::CargoCmd::Reboot(cmd) => cmd.execute(input.verbose),
        commands::CargoCmd::Upload(cmd) => cmd.execute(input.verbose),
    }
}
