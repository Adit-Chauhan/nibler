use handy_macros::exit_with;
use regex::Regex;
#[rustfmt::skip]
static BOTS: [&str; 50] = [
    "AFN|XDCC","ARUTHA-BATCH|1080p","ARUTHA-BATCH|720p","ARUTHA-BATCH|SD","ASource|Gerozaemon",
    "Arutha","Arutha|CPP","Arutha|DragonBall","Arutha|Naruto","Arutha|One-Piece",
    "Blargh|Cats","Blargh|Flep","Blargh|Other",
    "CHK|OP-Dump","CR-ARUTHA-IPv6|NEW","CR-ARUTHA|NEW","CR-HOLLAND-IPv6|NEW",
    "CR-HOLLAND|NEW","Chinese-Cartoons","Cthuko|Furuichi",
    "E-D|Raphtalia","Fincayra","Frostii|Tiger",
    "Ghouls|Arutha","Ginpachi-Sensei","Gin|TV","Hatsu|Arutha","HnG|Arutha",
    "Illum","K-F|Arutha","L-E|Ayukawa","L-E|Chiko","L-E|Yawara",
    "NIBL|Asian","O-L|Releases","Orphan|Arutha","RawManga","Retrofit|Filo",
    "SaberLily","Saizen|Arutha","Stardust|Kaoru","THORA|Arutha","[CMS]Shinobu",
    "[DCTP]Arutha","[FFF]Arutha","[Migoto]Kobato","[Oyatsu]Sena",
    "moviebox","pcela-anime|BiriBiri","tvbox"
];

#[derive(Debug, PartialEq, Eq)]
pub enum Args {
    Query { search: String },
    Direct { bot: String, packs: Vec<String> },
}

fn parse_direct(command: &[String]) -> Args {
    if command.len() != 2 {
        exit_with!("Error: Insufficient Arguments");
    }
    let result = BOTS.into_iter().filter(|x| &&command[0] == x).next();
    let bot: &str = match result {
        None => exit_with!("Error: Bot Not Found"),
        Some(botz) => botz,
    };
    let re: Vec<String> = Regex::new(r#"\d+"#)
        .expect("Regex Error")
        .find_iter(&command[1])
        .map(|x| x.as_str().to_string())
        .collect();

    Args::Direct {
        bot: bot.to_string(),
        packs: re,
    }
}

pub fn parse_args(args: &Vec<String>) -> Args {
    if args.len() < 2 {
        exit_with!("Error: Insufficient Arguments");
    }

    match args[0].to_ascii_lowercase().as_str() {
        "query" | "search" | "find" => {
            let joint = args[1..].join("+").replace(" ", "+");
            Args::Query { search: joint }
        }
        "direct" => parse_direct(&args[1..]),
        _ => exit_with!("Error: Invalid Input Argument"),
    }
}
