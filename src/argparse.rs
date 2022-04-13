use crate::CustomErrors;
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

//fn from(text: &str) -> Result<Args, ArgErr> {
//    match text.to_ascii_lowercase().as_str() {
//        "query" => Ok(Args::Query),
//        "search" => Ok(Args::Query),
//        "find" => Ok(Args::Query),
//        "direct" => Ok(Args::Direct),
//        _ => Err(ArgErr::IncorrectArgument),
//    }
//}

fn parse_direct(command: &[String]) -> Result<Args, CustomErrors> {
    if command.len() != 2 {
        return Err(CustomErrors::NumberOfArguments);
    }
    let result = BOTS.into_iter().filter(|x| &&command[0] == x).next();
    let bot: &str;
    match result {
        None => return Err(CustomErrors::BotNotFound),
        Some(botz) => bot = botz,
    };
    let re: Vec<String> = Regex::new(r#"\d+"#)
        .map_err(|_| CustomErrors::RegexError)?
        .find_iter(&command[1])
        .map(|x| x.as_str().to_string())
        .collect();

    Ok(Args::Direct {
        bot: bot.to_string(),
        packs: re,
    })
}

pub fn parse_args(args: &Vec<String>) -> Result<Args, CustomErrors> {
    if args.len() < 2 {
        return Err(CustomErrors::NumberOfArguments);
    }

    match args[0].to_ascii_lowercase().as_str() {
        "query" | "search" | "find" => Ok(Args::Query {
            search: args[1..].join(" "),
        }),
        "direct" => parse_direct(&args[1..]),
        _ => Err(CustomErrors::IncorrectArgument),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! svec {
	     ($($x:expr),*) => (vec![$($x.to_string()),*]);

    }
    #[test]
    fn test_arg_parse_initial_less_quantity() {
        assert_eq!(
            parse_args(&svec!["search"]),
            Err(CustomErrors::NumberOfArguments)
        );
    }
    #[test]
    fn test_arg_parse_search_terms() {
        let res = Ok(Args::Query {
            search: "it is working".to_string(),
        });
        // option is case insensitive
        assert_eq!(parse_args(&svec!["find", "it", "is", "working"]), res);
        assert_eq!(parse_args(&svec!["fINd", "it", "is", "working"]), res);
        assert_eq!(parse_args(&svec!["Search", "it", "is", "working"]), res);
        assert_eq!(parse_args(&svec!["Query", "it", "is", "working"]), res);
        assert_eq!(parse_args(&svec!["QuEry", "it", "is", "working"]), res);
        assert_eq!(parse_args(&svec!["QueRy", "it", "is", "working"]), res);
        // Search terms are not
        assert_ne!(parse_args(&svec!["QueRy", "it", "is", "woRking"]), res);
        assert_ne!(parse_args(&svec!["QueRy", "it", "Is", "working"]), res);
        assert_ne!(parse_args(&svec!["QueRy", "iT", "is", "working"]), res);
    }

    #[test]
    fn test_arg_parse_direct_incorrect_argument() {
        assert_eq!(
            parse_args(&svec!["Garbage", "It really Doesnt", "Matter"]),
            Err(CustomErrors::IncorrectArgument)
        );
    }
    #[test]
    fn test_arg_parse_direct_wrong_quantity_of_arguments() {
        // Less Args
        assert_eq!(
            parse_args(&svec!["direct", "Arutha"]),
            Err(CustomErrors::NumberOfArguments)
        );
        // More Args
        assert_eq!(
            parse_args(&svec!["direct", "Arutha", "Luffy", "Zoro"]),
            Err(CustomErrors::NumberOfArguments)
        );
    }
    #[test]
    fn test_arg_parse_direct_bad_bot() {
        assert_eq!(
            parse_args(&svec!["direct", "bad", "lamo"]),
            Err(CustomErrors::BotNotFound)
        );
    }

    #[test]
    fn test_arg_parse_direct_single() {
        assert_eq!(
            parse_args(&svec!["direct", "Arutha", "123"]),
            Ok(Args::Direct {
                bot: "Arutha".to_string(),
                packs: svec!["123"]
            })
        );
    }

    #[test]
    fn test_arg_parse_direct_multi_pack() {
        assert_eq!(
            parse_args(&svec!["dirEct", "Arutha", "123,456,1122"]),
            Ok(Args::Direct {
                bot: "Arutha".to_string(),
                packs: svec!["123", "456", "1122"]
            })
        );
    }
}
