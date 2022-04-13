use regex::Regex;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, TcpStream};
use std::str::from_utf8;
use std::thread;

use crate::CustomErrors;

lazy_static::lazy_static! {
     static ref DCC_SEND_REGEX: Regex =
        Regex::new(r#"DCC SEND "?(.*)"? (\d+) (\d+) (\d+)"#).unwrap();
    static ref PING_REGEX: Regex = Regex::new(r#"^PING :\d+"#).unwrap();
    static ref JOIN_REGEX: Regex = Regex::new(r#"JOIN :#.*"#).unwrap();
}

pub fn connect(name: &str) -> Result<TcpStream, CustomErrors> {
    let mut stream =
        TcpStream::connect("irc.rizon.net:6667").map_err(|_| CustomErrors::FailedToSetTCPStream)?;
    println!("Sending name");
    stream
        .write(format!("NICK {}\r\n", name).as_bytes())
        .map_err(|_| CustomErrors::FailedToSetName)?;
    println!("Sending name 2");
    stream
        .write(format!("USER {} * * :{}\r\n", name, name).as_bytes())
        .map_err(|_| CustomErrors::FailedToSetName)?;
    let mut message_buffer = String::new();
    loop {
        let message = read_line(&mut stream, &mut message_buffer);
        if message.is_err() {
            return Err(CustomErrors::ErrorReadingTcpStream);
        }
        let message = message.unwrap();
        println!("{:?}", message);
        if PING_REGEX.is_match(&message) {
            println!("regex Match");
            stream
                .write(message.replace("PING", "PONG").as_bytes())
                .unwrap();
            println!("Ponged");
            stream.write("JOIN #nibl\r\n".as_bytes()).unwrap();
            break;
        }
    }

    Ok(stream)
}

fn read_line(
    stream: &mut TcpStream,
    message_builder: &mut String,
) -> Result<String, std::io::Error> {
    let mut buffer = [0; 4];
    while !message_builder.contains("\n") {
        let count = stream.read(&mut buffer[..])?;
        message_builder.push_str(from_utf8(&buffer[..count]).unwrap_or_default());
    }
    let endline_offset = message_builder.find('\n').unwrap() + 1;
    let message = message_builder.get(..endline_offset).unwrap().to_string();
    message_builder.replace_range(..endline_offset, "");
    Ok(message)
}
