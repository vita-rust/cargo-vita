use std::{
    io::{Read, Write},
    net::TcpStream,
};

use anyhow::Context;
use colored::Colorize;
use log::info;

pub fn nc(ip: &str, port: u16, command: &str) -> anyhow::Result<()> {
    info!("{} {ip}:{port} -> {command}", "Sending command to".blue());

    let mut stream =
        TcpStream::connect((ip, port)).context("Unable to connect to command server")?;
    let command = format!("{command}\n");
    stream
        .write_all(command.as_bytes())
        .context("Unable to write to TCP socket")?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .context("Unable to read output")?;

    info!("{}: {}", "Server response".yellow(), response);

    Ok(())
}
