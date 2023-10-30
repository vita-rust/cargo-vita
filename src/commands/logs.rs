use std::{io::Read, net::TcpListener};

use anyhow::Context;
use clap::Args;
use colored::Colorize;
use log::{debug, error, info};

use super::Executor;

#[derive(Args, Debug)]
pub struct Logs {
    #[arg(long, short = 'p', env = "VITA_LOG_PORT", default_value_t = 8888)]
    port: u16,
}

impl Executor for Logs {
    fn execute(&self) -> anyhow::Result<()> {
        info!("{} {}", "Starting TCP server on port".blue(), self.port);

        let listener =
            TcpListener::bind(("0.0.0.0", self.port)).context("Unable to start TCP server")?;

        for stream in listener.incoming() {
            match stream {
                Ok(mut client) => {
                    debug!(
                        "{}: {}",
                        "Accepted connection from".blue(),
                        client.peer_addr().context("Unable to get peer address")?
                    );

                    std::thread::spawn(move || {
                        let mut buffer = [0; 1024];
                        loop {
                            match client.read(&mut buffer) {
                                Ok(0) => {
                                    debug!("{}", "Client disconnected".blue());
                                    break;
                                }
                                Ok(bytes_read) => {
                                    print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]))
                                }
                                Err(e) => {
                                    error!("{}: {}", "Error reading from client", e);
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        Ok(())
    }
}
