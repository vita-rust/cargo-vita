use std::ops::Deref;

use anyhow::Context;
use colored::Colorize;
use ftp::FtpStream;

use crate::commands::ConnectionArgs;

pub fn connect(conn: &ConnectionArgs, verbose: u8) -> anyhow::Result<FtpStream> {
    let ip = conn.vita_ip.deref();
    let port = conn.ftp_port;

    if verbose > 0 {
        println!("{} {ip}:{port}", "Connecting to Vita FTP server:".blue())
    }

    let ftp = FtpStream::connect((ip, port)).context("Unable to connect to Vita FTP server")?;

    Ok(ftp)
}
