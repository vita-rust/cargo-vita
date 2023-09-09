use std::str::FromStr;

use cargo_metadata::MetadataCommand;
use clap::{Args, Parser, Subcommand};
use serde::Deserialize;

pub use logs::*;
pub use reboot::*;
pub use run::*;
pub use update::*;
pub use upload::*;

mod logs;
mod reboot;
mod run;
mod update;
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

#[derive(Args, Debug)]
pub struct TitleArgs {
    #[arg(long, short = 't')]
    title_name: String,
    /// An alphanumeric string of 9 characters.
    #[arg(long, short = 'i', value_parser = clap::value_parser!(TitleId))]
    title_id: TitleId,
}

#[derive(Args, Debug)]
pub struct ConnectionArgs {
    /// An IPv4 address of your Vita. If not specified, defaults to VITA_IP env variable
    #[arg(long, short = 'a')]
    vita_ip: Option<String>,
    #[arg(long, short = 'f', default_value_t = 1337)]
    ftp_port: u16,
    #[arg(long, short = 'c', default_value_t = 1338)]
    cmd_port: u16,
}

impl ConnectionArgs {
    pub fn get_vita_ip(&self) -> String {
        self.vita_ip
            .clone()
            .or(std::env::var("VITA_IP").ok())
            .expect(
                "Vita ip address must be either provided by a flag or by an VITA_IP env variable.",
            )
    }
}

/// Run a cargo command. COMMAND will be forwarded to the real
/// `cargo` with the appropriate arguments for the 3DS target.
///
/// If an unrecognized COMMAND is used, it will be passed through unmodified
/// to `cargo` with the appropriate flags set for the 3DS target.
#[derive(Subcommand, Debug)]
#[command(allow_external_subcommands = true)]
pub enum CargoCmd {
    Upload(Upload),
    Run(Run),
    Logs(Logs),
    Reboot(Reboot),
}

#[derive(Args, Debug, Clone)]
pub struct TitleIdArg {
    #[arg(long, short = 'i', value_parser = clap::value_parser!(TitleId))]
    title_id: Option<TitleId>,
}

#[derive(Clone, Debug)]
pub struct TitleId(pub String);

impl<'de> Deserialize<'de> for TitleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for TitleId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 9 {
            return Err(format!("Title ID must be 9 characters long"));
        }

        if !s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(format!("Title ID consist of alpha numeric characters only"));
        }

        if !s
            .chars()
            .next()
            .ok_or_else(|| "Title ID must not be empty")?
            .is_alphabetic()
        {
            return Err(format!("Title ID must start with an alphabetic character"));
        }

        Ok(Self(s.to_uppercase().to_string()))
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct PackageMetadata {
    pub title_id: Option<TitleId>,
    pub title_name: Option<String>,
}

pub fn parse_crate_metadata() -> PackageMetadata {
    let meta = MetadataCommand::new()
        .exec()
        .expect("Failed to get cargo metadata");

    if let Some(pkg) = meta.workspace_default_packages().first() {
        if let Some(metadata) = pkg.metadata.as_object() {
            if let Some(metadata) = metadata.get("vita") {
                return serde_json::from_value::<PackageMetadata>(metadata.clone())
                    .expect("Unable to deserialize `package.metadata.vita`");
            }
        }
    }

    Default::default()
}
