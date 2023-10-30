use std::{
    io,
    process::{Command, Stdio},
};

use anyhow::{bail, Context};
use clap::{Args, Subcommand};
use colored::Colorize;
use log::{info, warn};
use suppaftp::FtpError;
use tempfile::NamedTempFile;

use super::{ConnectionArgs, Executor};
use crate::{
    ftp,
    meta::{parse_crate_metadata, VITA_TARGET},
};

#[derive(Args, Debug)]
pub struct Coredump {
    #[command(subcommand)]
    cmd: CoredumpCmd,
}

#[derive(Subcommand, Debug)]
pub enum CoredumpCmd {
    /// Downloads the latest coredump from Vita and uses vita-parse-core tool to parse it against the elf file of the current project.
    Parse(Parse),
    /// Deletes all coredump files from Vita
    Clean(Clean),
}

#[derive(Args, Debug)]
pub struct Parse {
    /// A path to the ELF file. If not provided the tool will try to guess it.
    #[arg(long)]
    elf: Option<String>,
    /// If ELF file is not explicitly provided, will use the artifact from this profile.
    #[arg(long, short = 'p', default_value = "debug")]
    profile: String,
    /// If true, will save coredump to tmp. Otherwise coredump is not saved to disk.
    #[arg(long, short = 's', default_value = "false")]
    persist: bool,
    #[command(flatten)]
    connection: ConnectionArgs,
}

#[derive(Args, Debug)]
pub struct Clean {
    #[command(flatten)]
    connection: ConnectionArgs,
}

impl Executor for Coredump {
    fn execute(&self) -> anyhow::Result<()> {
        match &self.cmd {
            CoredumpCmd::Parse(args) => {
                let mut ftp = ftp::connect(&args.connection)?;

                ftp.cwd("ux0:/data/")
                    .context("Unable to cwd to ux0:/data/")?;
                let files = ftp.list(None).context("Unable to list files in cwd")?;

                if let Some(coredump) = find_core_dumps(&files).max() {
                    info!("{}: {coredump}", "Downloading file".blue());
                    let mut reader = ftp
                        .retr_as_buffer(coredump)
                        .context("Unable to download coredump")?;

                    let mut tmp_file =
                        NamedTempFile::new().context("Unable to create temporary file")?;

                    io::copy(&mut reader, &mut tmp_file)
                        .context("Unable to write coredump to file")?;

                    let tmp_file = tmp_file.into_temp_path();

                    let path = if args.persist {
                        let file = tmp_file
                            .parent()
                            .context("Unable to get parent directory")?
                            .join(coredump);

                        tmp_file
                            .persist(&file)
                            .context("Unable to persist coredump")?;
                        file
                    } else {
                        tmp_file.to_path_buf()
                    };

                    let elf = match &args.elf {
                        Some(elf) => elf.clone(),
                        None => {
                            let (_, pkg, target_directory) = parse_crate_metadata(None)?;
                            let pkg = pkg.context("Not in a crate")?;

                            target_directory
                                .join(VITA_TARGET)
                                .join(&args.profile)
                                .join(pkg.name)
                                .with_extension("elf")
                                .to_string()
                        }
                    };

                    let mut command = Command::new("vita-parse-core");

                    command
                        .arg(&path)
                        .arg(&elf)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit());

                    info!("{}: {command:?}", "Parsing coredump".blue());

                    if !command.status()?.success() {
                        bail!("vita-parse-core failed");
                    }
                } else {
                    warn!("{}", "No coredump files found.".yellow())
                }
            }
            CoredumpCmd::Clean(args) => {
                let mut ftp = ftp::connect(&args.connection)?;
                ftp.cwd("ux0:/data/")
                    .context("Unable to cwd to ux0:/data/")?;

                let files = ftp.list(None).context("Unable to list files in cwd")?;
                let mut counter = 0;

                for file in find_core_dumps(&files) {
                    counter += 1;
                    info!("{}: {file}", "Deleting file".blue());

                    match ftp.rm(file) {
                        Ok(_) => {}
                        Err(FtpError::UnexpectedResponse(e))
                            if String::from_utf8_lossy(&e.body).contains("226 File deleted") => {}
                        Err(e) => return Err(e).context("Unable to delete file"),
                    }
                }

                if counter == 0 {
                    warn!("{}", "No coredump files found.".yellow())
                }
            }
        }

        Ok(())
    }
}

fn find_core_dumps(files: &[String]) -> impl Iterator<Item = &str> {
    files
        .iter()
        .filter_map(|line| line.split(' ').last())
        .filter(|file| file.starts_with("psp2core-") && file.ends_with(".bin.psp2dmp"))
}
