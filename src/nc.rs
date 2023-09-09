use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn nc(verbose: u8, ip: &str, port: u16, command: &str) {
    if verbose > 0 {
        println!("Sending command to {ip}:{port}: {command}");
    }
    let mut stream = TcpStream::connect((ip, port)).expect("Unable to connect to {ip}:{port}");
    let command = format!("{}\n", command);
    stream
        .write_all(command.as_bytes())
        .expect("Unable to write to TCP socket");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("Unable to read output");
    if verbose > 0 {
        println!("Server response: {}", response);
    }
}
