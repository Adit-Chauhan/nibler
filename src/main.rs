use rand::Rng;

use crate::irc::connect;

mod argparse;
mod irc;

#[derive(Debug, PartialEq, Eq)]
pub enum CustomErrors {
    BotNotFound,
    ErrorInParsingPacks,
    RegexError,
    IncorrectArgument,
    NumberOfArguments,
    FailedToSetName,
    FailedToSetTCPStream,
    ErrorReadingTcpStream,
}

fn main() {
    let ret = main_();
    if ret.is_ok() {
        return;
    }
    println!("Error {:?}", ret);
}

fn main_() -> Result<(), CustomErrors> {
    // Name
    let name: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .filter(|x| x.is_ascii_alphabetic())
        .take(10)
        .map(char::from)
        .collect();
    println!("Connecting using name \"{}\"", name);
    connect(&name)?;
    Ok(())
}
