use std::{fs::File, io::BufReader, path::Path};

use anyhow::{bail, Context};
use clap::Args;
use colored::Colorize;
use log::{debug, info};
use suppaftp::FtpError;
use walkdir::WalkDir;

use crate::ftp;

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
    fn execute(&self) -> anyhow::Result<()> {
        let source = Path::new(&self.source);
        if !source.exists() {
            bail!("Source path does not exist");
        }

        let mut ftp = ftp::connect(&self.connection)?;

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
            info!(
                "{} {source:?} {} {destination}",
                "Uploading".blue(),
                "to".blue(),
            );

            ftp.put_file(
                &destination,
                &mut BufReader::new(File::open(source).context("Unable to open source file")?),
            )
            .context("Uploading file failed")?;
        } else if source.is_dir() {
            for file in WalkDir::new(source)
                .into_iter()
                .filter_map(std::result::Result::ok)
            {
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
                    info!(
                        "{} {source_path:?} {} {destination}",
                        "Uploading".blue(),
                        "to".blue(),
                    );

                    ftp.put_file(
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
                    // Some of these errors are benign (e.g. when a directory already exists),
                    // so if an error happens it does not return Err, just print a debug log.
                    if ftp.cwd(&destination).is_err() {
                        info!("{} {destination}", "Creating directory".blue());
                        match ftp.mkdir(&destination) {
                            Ok(()) => {}
                            Err(FtpError::UnexpectedResponse(e))
                                if String::from_utf8_lossy(&e.body)
                                    .starts_with("226 Directory created.") => {}
                            Err(e) => {
                                debug!(
                                    "{}: {destination}, {e}",
                                    "Unable to create directory ".red()
                                );
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
