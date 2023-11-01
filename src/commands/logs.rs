use std::{
    io::{Cursor, Read},
    net::{Ipv4Addr, TcpListener},
    str::FromStr,
};

use anyhow::{bail, Context};
use clap::{Args, Subcommand};
use colored::Colorize;
use log::{debug, error, info};

use crate::ftp;

use super::{ConnectionArgs, Executor};

#[derive(Args, Debug)]
pub struct Logs {
    #[command(subcommand)]
    cmd: Option<LogsCmd>,

    #[arg(long, short = 'p', env = "VITA_LOG_PORT", default_value_t = 8888)]
    port: u16,
}

#[derive(Subcommand, Debug)]
pub enum LogsCmd {
    /// Start a TCP server on 0.0.0.0 and print to stdout all bytes read from the socket
    Listen,
    /// Reconfigures PrincessLog via vita-companion.
    /// This will upload the configuration file with the ip address of your host and a port to your Vita.
    Configure(Configure),
}

#[derive(Args, Debug)]
pub struct Configure {
    #[command(flatten)]
    pub connection: ConnectionArgs,

    #[arg(long)]
    host_ip_address: Option<String>,

    #[arg(long, default_value = "false")]
    kernel_debug: bool,
}

static MAGIC: &[u8] = b"NLM\0";
static NLM_CONFIG_FLAGS_BIT_QAF_DEBUG_PRINTF: u32 = 1 << 0;

impl Logs {
    fn configure(&self, configure: &Configure, verbose: u8) -> anyhow::Result<()> {
        let filename = "ur0:/data/NetLoggingMgrConfig.bin";
        println!(
            "{} {filename}",
            "Downloading the existing config from".blue()
        );
        let mut ftp = ftp::connect(&configure.connection, verbose)?;

        let file = ftp.retr_as_buffer(filename);
        if let Ok(mut config) = file {
            println!("{}", "Found existing config".blue());
            let mut buffer = [0; 4];
            config.read_exact(&mut buffer)?;
            if buffer != MAGIC {
                println!("{}", "Existing config is invalid".red());
            }

            config.read_exact(&mut buffer)?;
            let ip = Ipv4Addr::from(buffer);
            config.read_exact(&mut buffer)?;
            let flags = u32::from_le_bytes(buffer);

            let mut buffer = [0; 2];
            config.read_exact(&mut buffer)?;
            let port = u16::from_le_bytes(buffer);

            println!("{}: {ip}:{port}", "Current log address".yellow());
            let kdbg = (flags & NLM_CONFIG_FLAGS_BIT_QAF_DEBUG_PRINTF) != 0;
            println!("{}: {kdbg}", "Kernel debug print".yellow());
        }

        let ip = match &configure.host_ip_address {
            Some(ip) => Ipv4Addr::from_str(ip)?,
            None => match local_ip_address::local_ip()? {
                std::net::IpAddr::V4(ip) => ip,
                std::net::IpAddr::V6(_) => bail!("Unable to guess host ip address"),
            },
        };

        println!(
            "{} {ip}:{port} {} {kdbg}",
            "Setting log address to".blue(),
            "and kernel debug print to".blue(),
            port = self.port,
            kdbg = configure.kernel_debug,
        );

        let mut config = Vec::new();
        config.extend_from_slice(MAGIC);
        config.extend_from_slice(&ip.octets());

        let flags = match configure.kernel_debug {
            true => NLM_CONFIG_FLAGS_BIT_QAF_DEBUG_PRINTF,
            false => 0,
        };

        config.extend_from_slice(&flags.to_le_bytes());
        config.extend_from_slice(&self.port.to_le_bytes());
        config.extend_from_slice(&[0, 0]);

        print!("{} {}", "Saving config to".blue(), filename);
        let _ = ftp.mkdir("ur0:/data/");
        ftp.put_file(filename, &mut Cursor::new(config))?;

        println!("Config will take effect after Vita is rebooted");

        Ok(())
    }
    fn listen(&self, verbose: u8) -> anyhow::Result<()> {
        if verbose > 0 {
            println!("{} {}", "Starting TCP server on port".blue(), self.port);
        }

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

impl Executor for Logs {
    fn execute(&self, verbose: u8) -> anyhow::Result<()> {
        match self.cmd.as_ref() {
            Some(LogsCmd::Configure(cmd)) => self.configure(cmd, verbose),
            Some(LogsCmd::Listen) | None => self.listen(verbose),
        }
    }
}
