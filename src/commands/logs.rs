use std::{
    io::{Cursor, Read},
    net::{Ipv4Addr, TcpListener},
    str::FromStr,
};

use anyhow::{bail, Context};
use clap::{Args, Subcommand};
use colored::Colorize;
use log::{debug, error, info, warn};

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
    /// Reconfigures `PrincessLog` via vitacompanion.
    /// This will upload the configuration file with the ip address of your host and a port to your Vita.
    Configure(Configure),
}

#[derive(Args, Debug)]
pub struct Configure {
    #[command(flatten)]
    pub connection: ConnectionArgs,

    /// The IP address of the host to send logs to.
    /// Vita will connect to this address via TCP write logs to the socket.
    #[arg(long)]
    host_ip_address: Option<String>,

    /// Enabled kernel debug logs.
    #[arg(long, default_value = "false")]
    kernel_debug: bool,
}

static MAGIC: [u8; 4] = [b'N', b'L', b'M', 0];
static NLM_CONFIG_FLAGS_BIT_QAF_DEBUG_PRINTF: u32 = 1 << 0;

struct PrincessLogConfig {
    magic: [u8; 4],
    ip: Ipv4Addr,
    port: u16,
    kernel_debug: bool,
}

impl PrincessLogConfig {
    fn new(ip: Ipv4Addr, port: u16, kernel_debug: bool) -> Self {
        Self {
            magic: MAGIC,
            ip,
            port,
            kernel_debug,
        }
    }

    fn parse<R: Read>(config: &mut R) -> anyhow::Result<Self> {
        let mut magic = [0; 4];
        config.read_exact(&mut magic)?;

        let mut buffer = [0; 4];
        config.read_exact(&mut buffer)?;
        let ip = Ipv4Addr::from(buffer);
        config.read_exact(&mut buffer)?;
        let flags = u32::from_le_bytes(buffer);
        let mut buffer = [0; 2];
        config.read_exact(&mut buffer)?;
        let port = u16::from_le_bytes(buffer);

        let kernel_debug = (flags & NLM_CONFIG_FLAGS_BIT_QAF_DEBUG_PRINTF) != 0;

        Ok(Self {
            magic,
            ip,
            port,
            kernel_debug,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let flags = if self.kernel_debug {
            NLM_CONFIG_FLAGS_BIT_QAF_DEBUG_PRINTF
        } else {
            0
        };

        let mut res = Vec::with_capacity(16);
        res.extend_from_slice(&MAGIC);
        res.extend_from_slice(&self.ip.octets());
        res.extend_from_slice(&flags.to_le_bytes());
        res.extend_from_slice(&self.port.to_le_bytes());
        res.extend_from_slice(&[0, 0]);

        res
    }
}

impl Logs {
    fn configure(&self, configure: &Configure) -> anyhow::Result<()> {
        let filename = "ur0:/data/NetLoggingMgrConfig.bin";
        debug!(
            "{} {filename}",
            "Downloading the existing config from".blue()
        );
        let mut ftp = ftp::connect(&configure.connection)?;

        match ftp.retr_as_buffer(filename) {
            Ok(mut file) => {
                info!("{}", "Found existing config".blue());

                match PrincessLogConfig::parse(&mut file) {
                    Ok(c) => {
                        if c.magic == MAGIC {
                            info!(
                                "{} {ip}:{port} {} {kdbg}",
                                "Existing config has address".yellow(),
                                "and kernel debug print is".yellow(),
                                ip = c.ip,
                                port = c.port,
                                kdbg = c.kernel_debug
                            );
                        } else {
                            warn!("{}", "Existing config has invalid magic".red());
                        }
                    }

                    Err(err) => {
                        warn!("{}: {err}", "Failed to parse existing config".red());
                    }
                }
            }
            Err(err) => {
                warn!("{}: {err}", "Failed to download existing config".red());
            }
        }

        let ip = match &configure.host_ip_address {
            Some(ip) => Ipv4Addr::from_str(ip)?,
            None => match local_ip_address::local_ip()? {
                std::net::IpAddr::V4(ip) => ip,
                std::net::IpAddr::V6(_) => bail!("Unable to guess host ip address"),
            },
        };

        info!(
            "{} {ip}:{port} {} {kdbg}",
            "Setting log address to".blue(),
            "and kernel debug print to".blue(),
            port = self.port,
            kdbg = configure.kernel_debug,
        );

        info!("{} {}", "Saving config to".blue(), filename);
        let _ = ftp.mkdir("ur0:/data/");
        let cfg = PrincessLogConfig::new(ip, self.port, configure.kernel_debug).serialize();
        ftp.put_file(filename, &mut Cursor::new(cfg))?;

        info!(
            "{}",
            "Config will take effect after Vita is rebooted".yellow()
        );

        Ok(())
    }
    fn listen(&self) -> anyhow::Result<()> {
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
                                    print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
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
    fn execute(&self) -> anyhow::Result<()> {
        match self.cmd.as_ref() {
            Some(LogsCmd::Configure(cmd)) => self.configure(cmd),
            Some(LogsCmd::Listen) | None => self.listen(),
        }
    }
}
