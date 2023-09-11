use std::{
    io::{Read, Write},
    net::TcpStream,
};

use anyhow::Context;
use colored::Colorize;

pub fn nc(verbose: u8, ip: &str, port: u16, command: &str) -> anyhow::Result<()> {
    if verbose > 0 {
        println!("{} {ip}:{port} -> {command}", "Sending command to".blue());
    }
    let mut stream =
        TcpStream::connect((ip, port)).context("Unable to connect to command server")?;
    let command = format!("{}\n", command);
    stream
        .write_all(command.as_bytes())
        .context("Unable to write to TCP socket")?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .context("Unable to read output")?;
    if verbose > 0 {
        println!("{} {}", "Server response:".yellow(), response);
    }

    Ok(())
}
