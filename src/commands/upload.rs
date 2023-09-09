use std::{fs::File, io::BufReader, ops::Deref, path::Path};

use cargo_metadata::MetadataCommand;
use clap::Args;
use ftp::FtpStream;

use super::ConnectionArgs;

#[derive(Args, Debug)]
pub struct Upload {
    #[command(flatten)]
    connection: ConnectionArgs,

    /// A path to a file or a directory. If a directory is passed it will be copied recursively.
    #[arg(long, short = 's')]
    source: String,

    /// A directory on Vita where a file will be saved. Slash in the end indicates that it's a directory.
    #[arg(long, short = 'd', default_value = "ux0:/download/")]
    destination: String,
}

impl Upload {
    pub fn execute(&self, verbose: u8) {
        let source = Path::new(&self.source);
        if !source.exists() {
            panic!("Source path does not exist");
        }

        let ip = self.connection.get_vita_ip();

        if verbose > 0 {
            println!("Connecting to {ip}:{}", self.connection.ftp_port);
        }

        let mut ftp = FtpStream::connect((ip.deref(), self.connection.ftp_port)).unwrap();

        let destination = if self.destination.chars().last() == Some('/') {
            format!(
                "{}{}",
                self.destination,
                source
                    .file_name()
                    .expect("Unable to get source file name")
                    .to_string_lossy()
            )
        } else {
            self.destination.clone()
        };

        if source.is_file() {
            if verbose > 0 {
                println!("Uploading {source:?} to {destination}");
            }

            ftp.put(
                &destination,
                &mut BufReader::new(File::open(source).expect("Unable to open source file")),
            )
            .expect("Uploading file failed");
        } else if source.is_dir() {
            walkdir::WalkDir::new(source)
                .into_iter()
                .filter_map(|e| e.ok())
                .for_each(|e| {
                    let source_path = e.path();

                    let destination = format!(
                        "{}/{}",
                        destination,
                        source_path
                            .strip_prefix(source)
                            .expect("Unable to strip source path prefix")
                            .to_string_lossy()
                    );

                    if e.file_type().is_file() {
                        if verbose > 0 {
                            println!("Uploading {source_path:?} to {destination}",);
                        }

                        ftp.put(
                            &destination,
                            &mut BufReader::new(
                                File::open(source_path).expect("Unable to open source file"),
                            ),
                        )
                        .expect("Uploading file failed");
                    } else if e.file_type().is_dir() {
                        if ftp.pwd().ok().as_deref() == Some(&destination) {
                            return;
                        }

                        // let _ = ftp.quit();
                        // ftp = FtpStream::connect((ip.deref(), self.connection.ftp_port)).unwrap();

                        // For some reason doing multiple cwd in a single connection breaks vitacompanion,
                        // So we'll skip directory creation errors.
                        if ftp.cwd(&destination).is_err() {
                            if verbose > 0 {
                                println!("Creating directory {destination}");
                            }
                            match ftp.mkdir(&destination) {
                                Ok(_) => {}
                                Err(ftp::FtpError::InvalidResponse(e))
                                    if e == "226 Directory created." => {}
                                Err(e) => {
                                    if verbose > 0 {
                                        println!("Unable to create directory: {destination}, {e}");
                                    }
                                }
                            };
                        }
                    }
                });
        } else {
            panic!("Unsupported source file type");
        }

        let _ = ftp.quit();
    }
}
