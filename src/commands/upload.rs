use std::{fs::File, io::BufReader, ops::Deref, path::Path};

use anyhow::{bail, Context};
use clap::Args;
use colored::Colorize;
use ftp::FtpStream;
use walkdir::WalkDir;

use super::{ConnectionArgs, Executor};

#[derive(Args, Debug)]
pub struct Upload {
    #[command(flatten)]
    pub connection: ConnectionArgs,

    /// A path to a file or a directory. If a directory is passed it will be copied recursively.
    #[arg(long, short = 's')]
    pub source: String,

    /// A directory on Vita where a file will be saved. Slash in the end indicates that it's a directory.
    #[arg(long, short = 'd', default_value = "ux0:/download/")]
    pub destination: String,
}

impl Executor for Upload {
    fn execute(&self, verbose: u8) -> anyhow::Result<()> {
        let source = Path::new(&self.source);
        if !source.exists() {
            bail!("Source path does not exist");
        }

        let ip = &self.connection.vita_ip;
        let port = self.connection.ftp_port;
        if verbose > 0 {
            println!("{} {ip}:{port}", "Connecting to Vita FTP:".blue(),);
        }

        let mut ftp =
            FtpStream::connect((ip.deref(), port)).context("Unable to connect to FTP server")?;

        let destination = if self.destination.ends_with('/') {
            format!(
                "{}{}",
                self.destination,
                source
                    .file_name()
                    .context("Unable to get source file name")?
                    .to_string_lossy()
            )
        } else {
            self.destination.clone()
        };

        if source.is_file() {
            if verbose > 0 {
                println!(
                    "{}",
                    format!("Uploading {source:?} to {destination}").blue()
                );
            }

            ftp.put(
                &destination,
                &mut BufReader::new(File::open(source).context("Unable to open source file")?),
            )
            .context("Uploading file failed")?;
        } else if source.is_dir() {
            for file in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
                let source_path = file.path();

                let destination = format!(
                    "{}/{}",
                    destination,
                    source_path
                        .strip_prefix(source)
                        .context("Unable to strip source path prefix")?
                        .to_string_lossy()
                );

                if file.file_type().is_file() {
                    if verbose > 0 {
                        println!(
                            "{}",
                            format!("Uploading {source_path:?} to {destination}").blue()
                        );
                    }

                    ftp.put(
                        &destination,
                        &mut BufReader::new(
                            File::open(source_path).context("Unable to open source file")?,
                        ),
                    )
                    .context("Uploading file failed")?;
                } else if file.file_type().is_dir() {
                    if ftp.pwd().ok().as_deref() == Some(&destination) {
                        continue;
                    }

                    // For some reason doing multiple cwd in a single connection breaks vitacompanion,
                    // So we'll skip directory creation errors.
                    if ftp.cwd(&destination).is_err() {
                        if verbose > 0 {
                            println!("{} {destination}", "Creating directory".blue());
                        }
                        match ftp.mkdir(&destination) {
                            Ok(_) => {}
                            Err(ftp::FtpError::InvalidResponse(e))
                                if e.starts_with("226 Directory created.") => {}
                            Err(e) => {
                                if verbose > 1 {
                                    eprintln!(
                                        "{} {destination}, {e}",
                                        "Unable to create directory: ".red()
                                    );
                                }
                            }
                        };
                    }
                }
            }
        } else {
            bail!("Unsupported source file type");
        }

        let _ = ftp.quit();

        Ok(())
    }
}
