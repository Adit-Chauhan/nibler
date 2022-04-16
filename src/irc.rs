use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, trace};
use regex::{Captures, Regex};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, TcpStream};
use std::str::from_utf8;
use std::thread;

lazy_static::lazy_static! {
     static ref DCC_SEND_REGEX: Regex =
        Regex::new(r#"DCC SEND "?(.*)"? (\d+) (\d+) (\d+)"#).unwrap();
    static ref PING_REGEX: Regex = Regex::new(r#"^PING :\d+"#).unwrap();
    static ref JOIN_REGEX: Regex = Regex::new(r#"JOIN :#.*"#).unwrap();
}
#[derive(Debug)]
struct ParesedDCC {
    file: String,
    ip: String,
    port: String,
    size: usize,
}
impl ParesedDCC {
    fn new(caps: Captures) -> Self {
        let mut s = caps[1].to_string();
        if s.as_bytes()[s.len() - 1] == '\"' as u8 {
            s.pop();
        }
        ParesedDCC {
            file: s,
            ip: caps[2].to_string(),
            port: caps[3].to_string(),
            size: caps[4].parse::<usize>().unwrap(),
        }
    }
}

pub fn connect(name: &str) -> Result<TcpStream, Box<dyn Error>> {
    let mut stream = TcpStream::connect("irc.rizon.net:6667")?;
    stream.write(format!("NICK {}\r\n", name).as_bytes())?;
    debug!("Set NICK");
    stream.write(format!("USER {} * * :{}\r\n", name, name).as_bytes())?;
    debug!("Set USER");
    let mut message_buffer = String::new();
    loop {
        let message = read_line(&mut stream, &mut message_buffer)?;
        trace!("{}", &message);
        if PING_REGEX.is_match(&message) {
            debug!("Ping Match");
            stream.write(message.replace("PING", "PONG").as_bytes())?;
            stream.write("JOIN #nibl\r\n".as_bytes())?;
            break;
        }
    }

    Ok(stream)
}

pub fn download_packs(
    stream: &mut TcpStream,
    bot: &str,
    packs: &Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let main_bar = indicatif::MultiProgress::new();
    for pack in packs {
        stream.write(format!("PRIVMSG {} :xdcc send #{}\r\n", bot, pack).as_bytes())?;
    }
    let mut downloads = Vec::new();
    let mut message_builder = String::new();
    while downloads.len() < packs.len() {
        let message = read_line(stream, &mut message_builder)?;
        trace!("{}", message);
        if PING_REGEX.is_match(&message) {
            debug!("Ping Match");
            stream.write(message.replace("PING", "PONG").as_bytes())?;
        }
        if !DCC_SEND_REGEX.is_match(&message) {
            continue;
        }

        let caps = DCC_SEND_REGEX.captures(&message).unwrap();
        let dcc = ParesedDCC::new(caps);
        let pb = main_bar.insert(downloads.len(), ProgressBar::new(dcc.size as u64));
        pb.set_style(ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("#>-"));
        downloads.push(thread::spawn(move || download_single(pb, dcc)))
    }
    stream.write("QUIT :adios\r\n".as_bytes())?;
    stream.shutdown(Shutdown::Both)?;
    downloads
        .into_iter()
        .for_each(|x| x.join().unwrap().unwrap());
    Ok(())
}

fn download_single(pb: ProgressBar, dcc: ParesedDCC) -> Result<(), std::io::Error> {
    let mut file = File::create(&dcc.file)?;
    let ip: IpAddr = IpAddr::V4(Ipv4Addr::from(dcc.ip.parse::<u32>().unwrap()));
    let mut stream = TcpStream::connect(format!("{}:{}", ip, dcc.port))?;
    let mut buffer = [0; 65536];
    let mut progress: usize = 0;
    pb.set_message(format!("{}", dcc.file));
    while progress < dcc.size {
        let count = stream.read(&mut buffer[..])?;
        file.write(&mut buffer[..count])?;
        progress += count;
        pb.set_position(progress as u64);
    }
    pb.finish_and_clear();
    stream.shutdown(Shutdown::Both)?;
    file.flush()?;
    Ok(())
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
