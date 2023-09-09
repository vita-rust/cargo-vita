use std::{io::Read, net::TcpListener};

use clap::Args;

#[derive(Args, Debug)]
pub struct Logs {
    #[arg(long, short = 'p', default_value_t = 8888)]
    port: u16,
}

impl Logs {
    pub fn execute(&self, verbose: u8) {
        if verbose > 0 {
            println!("Starting TCP server on port {}", self.port);
        }

        let listener =
            TcpListener::bind(("0.0.0.0", self.port)).expect("Unable to start TCP server");

        for stream in listener.incoming() {
            match stream {
                Ok(mut client) => {
                    if verbose > 1 {
                        println!(
                            ">>> Accepted connection from: {}",
                            client.peer_addr().expect("Unable to get peer address")
                        );
                    }

                    std::thread::spawn(move || {
                        let mut buffer = [0; 1024];
                        loop {
                            match client.read(&mut buffer) {
                                Ok(0) => {
                                    if verbose > 1 {
                                        println!(">>> Client disconnected");
                                    }
                                    break;
                                }
                                Ok(bytes_read) => {
                                    print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]))
                                }
                                Err(e) => {
                                    eprintln!("Error reading from client: {}", e);
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
    }
}
