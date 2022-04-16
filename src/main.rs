use crate::argparse::Args;
use crate::irc::{connect, download_packs};
use log::debug;
use rand::Rng;
use std::error::Error;
mod argparse;
mod irc;
mod list;
mod search;

fn main() {
    env_logger::init();
    let ret = main_();
    if ret.is_ok() {
        return;
    }
    println!("Error {:?}", ret);
}

fn main_() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args = crate::argparse::parse_args(&args)?;
    let args = match args {
        Args::Query { search: s } => crate::search::search_to_direct(&s),
        Args::Direct { bot, packs } => Args::Direct { bot, packs },
    };

    let name: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .filter(|x| x.is_ascii_alphabetic())
        .take(10)
        .map(char::from)
        .collect();
    println!("Connecting using name \"{}\"", name);
    let mut stream = connect(&name)?;
    debug!("Connected to server");
    if let Args::Direct { bot, packs } = args {
        download_packs(&mut stream, &bot, &packs)?;
    }
    Ok(())
}
