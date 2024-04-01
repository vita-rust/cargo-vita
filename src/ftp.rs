use anyhow::Context;
use colored::Colorize;
use log::info;
use suppaftp::FtpStream;

use crate::commands::ConnectionArgs;

pub fn connect(conn: &ConnectionArgs) -> anyhow::Result<FtpStream> {
    let ip = &*conn.vita_ip;
    let port = conn.ftp_port;

    info!("{} {ip}:{port}", "Connecting to Vita FTP server".blue());

    let ftp = FtpStream::connect((ip, port)).context("Unable to connect to Vita FTP server")?;

    Ok(ftp)
}
