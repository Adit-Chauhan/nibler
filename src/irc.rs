use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, trace};
use regex::{Captures, Regex};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::str::from_utf8;
use std::thread;

use crate::CustomErrors;

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

pub fn connect(name: &str) -> Result<TcpStream, CustomErrors> {
    let mut stream =
        TcpStream::connect("irc.rizon.net:6667").map_err(|_| CustomErrors::FailedToSetTCPStream)?;
    stream
        .write(format!("NICK {}\r\n", name).as_bytes())
        .map_err(|_| CustomErrors::FailedToSetName)?;
    debug!("Set NICK");
    stream
        .write(format!("USER {} * * :{}\r\n", name, name).as_bytes())
        .map_err(|_| CustomErrors::FailedToSetName)?;
    debug!("Set USER");
    let mut message_buffer = String::new();
    loop {
        let message = read_line(&mut stream, &mut message_buffer);
        if message.is_err() {
            return Err(CustomErrors::ErrorReadingTcpStream);
        }
        let message = message.unwrap();
        trace!("{}", &message);
        if PING_REGEX.is_match(&message) {
            debug!("Ping Match");
            stream
                .write(message.replace("PING", "PONG").as_bytes())
                .unwrap();
            stream.write("JOIN #nibl\r\n".as_bytes()).unwrap();
            break;
        }
    }

    Ok(stream)
}

pub fn download_packs(stream: &mut TcpStream, bot: &str, packs: &Vec<String>) {
    let main_bar = indicatif::MultiProgress::new();
    for pack in packs {
        stream
            .write(format!("PRIVMSG {} :xdcc send #{}\r\n", bot, pack).as_bytes())
            .expect("Failed to send xdcc request");
    }
    let mut downloads = Vec::new();
    let mut message_builder = String::new();
    // (index.done,total);
    //  let (tx, rx) = mpsc::channel();
    //tx.send((0_u16, 10_usize, 10_usize, "Filename".to_string()));
    while downloads.len() < packs.len() {
        let message = read_line(stream, &mut message_builder).unwrap();
        trace!("{}", message);
        if PING_REGEX.is_match(&message) {
            debug!("Ping Match");
            stream
                .write(message.replace("PING", "PONG").as_bytes())
                .unwrap();
        }
        if !DCC_SEND_REGEX.is_match(&message) {
            continue;
        }

        let caps = DCC_SEND_REGEX.captures(&message).unwrap();
        let dcc = ParesedDCC::new(caps);
        debug!("made dcc");
        debug!("{:#?}", dcc);
        //  let ttx = tx.clone();
        let dlen = downloads.len() as u16;
        let pb = main_bar.insert(dlen as usize, ProgressBar::new(dcc.size as u64));
        pb.set_style(ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("#>-"));
        downloads.push(thread::spawn(move || download_single(dlen, pb, dcc)))
    }
    stream.write("QUIT :adios\r\n".as_bytes()).unwrap();
    stream.shutdown(Shutdown::Both).unwrap();
    downloads
        .into_iter()
        .for_each(|x| x.join().unwrap().unwrap());
}

fn download_single(_index: u16, pb: ProgressBar, dcc: ParesedDCC) -> Result<(), std::io::Error> {
    let mut file = File::create(&dcc.file)?;
    let mut stream = TcpStream::connect(format!("{}:{}", dcc.ip, dcc.port))?;
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
