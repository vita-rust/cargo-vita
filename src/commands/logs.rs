use std::{io::Read, net::TcpListener};

use anyhow::Context;
use clap::Args;
use colored::Colorize;

use super::Executor;

#[derive(Args, Debug)]
pub struct Logs {
    #[arg(long, short = 'p', env = "VITA_LOG_PORT", default_value_t = 8888)]
    port: u16,
}

impl Executor for Logs {
    fn execute(&self, verbose: u8) -> anyhow::Result<()> {
        if verbose > 0 {
            println!("{} {}", "Starting TCP server on port".blue(), self.port);
        }

        let listener =
            TcpListener::bind(("0.0.0.0", self.port)).context("Unable to start TCP server")?;

        for stream in listener.incoming() {
            match stream {
                Ok(mut client) => {
                    if verbose > 1 {
                        println!(
                            "{}: {}",
                            "Accepted connection from".blue(),
                            client.peer_addr().context("Unable to get peer address")?
                        );
                    }

                    std::thread::spawn(move || {
                        let mut buffer = [0; 1024];
                        loop {
                            match client.read(&mut buffer) {
                                Ok(0) => {
                                    if verbose > 1 {
                                        println!("{}", "Client disconnected".blue());
                                    }
                                    break;
                                }
                                Ok(bytes_read) => {
                                    print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]))
                                }
                                Err(e) => {
                                    eprintln!("{}: {}", "Error reading from client".red(), e);
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }

        Ok(())
    }
}
