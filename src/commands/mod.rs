use clap::{Args, Parser, Subcommand};

pub use build::*;
use enum_dispatch::enum_dispatch;
pub use logs::*;
pub use reboot::*;
pub use run::*;
pub use upload::*;

use crate::meta::TitleId;

mod build;
mod logs;
mod reboot;
mod run;
mod upload;

#[derive(Parser, Debug)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum Cargo {
    #[command(name = "vita")]
    Input(Input),
}

#[derive(Args, Debug)]
#[command(version, about)]
pub struct Input {
    #[command(subcommand)]
    pub cmd: CargoCmd,

    /// Print the exact commands `cargo-vita` is running.
    /// Passing this flag multiple times will enable verbose mode for the rust compiler.
    #[arg(long, short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// Run a cargo command. COMMAND will be forwarded to the real
/// `cargo` with the appropriate arguments for the 3DS target.
///
/// If an unrecognized COMMAND is used, it will be passed through unmodified
/// to `cargo` with the appropriate flags set for the 3DS target.
#[enum_dispatch]
#[derive(Subcommand, Debug)]
#[command(allow_external_subcommands = true)]
pub enum CargoCmd {
    Build(Build),
    Upload(Upload),
    Run(Run),
    Logs(Logs),
    Reboot(Reboot),
}

#[enum_dispatch(CargoCmd)]
pub trait Executor {
    fn execute(&self, verbose: u8) -> anyhow::Result<()>;
}

#[derive(Args, Debug)]
pub struct TitleArgs {
    #[arg(long, short = 't')]
    title_name: String,
    /// An alphanumeric string of 9 characters.
    #[arg(long, short = 'i', value_parser = clap::value_parser!(TitleId))]
    title_id: TitleId,
}

#[derive(Args, Debug, Clone)]
pub struct ConnectionArgs {
    /// An IPv4 address of your Vita.
    #[arg(long, short = 'a', env = "VITA_IP")]
    vita_ip: String,
    #[arg(long, short = 'f', env = "VITA_FTP_PORT", default_value_t = 1337)]
    ftp_port: u16,
    #[arg(long, short = 'c', env = "VITA_CMD_PORT", default_value_t = 1338)]
    cmd_port: u16,
}

#[derive(Args, Debug, Clone)]
pub struct TitleIdArg {
    #[arg(long, short = 'i', value_parser = clap::value_parser!(TitleId))]
    title_id: Option<TitleId>,
}
