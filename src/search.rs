use crate::{argparse::Args, list::StatefulList};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use handy_macros::{iter_map, match_exit};
use regex::{Captures, Regex};
use reqwest::blocking::get;
use std::collections::HashMap;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame, Terminal,
};

const API_BASE: &str = "https://api.nibl.co.uk/nibl";

#[derive(Debug, Clone)]
struct SearchResultItem {
    bot_id: u16,
    pack: u32,
    name: String,
    size: String,
}
impl SearchResultItem {
    fn new(cap: Captures) -> Self {
        let a = match_exit!(
            cap[1].parse::<u16>(),
            "Failed To Parse Bot ID, Server Side Error, Please Retry"
        );
        let b = match_exit!(
            cap[2].parse::<u32>(),
            "Failed To Parse Pack ID, Server Side Error, Please Retry"
        );
        SearchResultItem {
            bot_id: a,
            pack: b,
            name: cap[3].to_string(),
            size: cap[4].to_string(),
        }
    }
}

struct App {
    items: StatefulList<SearchResultItem>,
    selected: Vec<usize>,
    in_bot: Option<u16>,
    download: bool,
}
impl App {
    fn filter_items(&mut self) {
        self.items = StatefulList::with_items(
            self.items
                .items
                .clone()
                .into_iter()
                .filter(|x| x.bot_id == self.in_bot.unwrap())
                .collect(),
        );
    }
}

pub fn search_to_direct(sterm: &str) -> Args {
    let res = search_disp(&sterm);
    if res.is_err() {
        println!("Error in searching");
        std::process::exit(0);
    }
    res.unwrap()
}

fn get_bots() -> Result<HashMap<u16, String>, reqwest::Error> {
    let a = get(format!("{}/bots", API_BASE))?.text()?;
    let re_id = Regex::new(r#""id":(\d+),"name":"(.+?)""#).expect("Failed To Parse Regex");
    let caps = re_id.captures_iter(&a);
    let mut map = HashMap::new();
    let _ = caps
        .into_iter()
        .map(|c| {
            map.insert(c[1].parse::<u16>().unwrap(), c[2].to_string());
        })
        .collect::<Vec<_>>();
    Ok(map)
}

fn search_disp(query: &str) -> Result<Args, Box<dyn Error>> {
    let resp = match_exit!(
        get(format!("{}/search?query={}", API_BASE, query)),
        "Failed To Connect to Website"
    );
    let resp = match_exit!(resp.text(), "Failed to get text from response");
    let search_re = Regex::new(r#""botId":(\d+),"number":(\d+),"name":"(.+?)","size":"(.+?)""#)
        .expect("Failed To parse Regex");

    let results: Vec<SearchResultItem> = search_re
        .captures_iter(&resp)
        .into_iter()
        .map(|x| SearchResultItem::new(x))
        .collect();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    // create app and run it
    let app = App {
        items: StatefulList::with_items(results),
        selected: Vec::new(),
        in_bot: None,
        download: false,
    };
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
        return Err(err);
    }
    let bot_name = |id: u16| -> String {
        let map = match_exit!(get_bots(), "Failed To Connect To site");
        let name = map.get(&id).expect("Should not be possible");
        name.to_string()
    };
    let a = res.unwrap();
    if !a.download {
        println!("nothing to download");
        std::process::exit(0);
    }
    let s: Vec<String> = iter_map!(into a.selected,|x| x.to_string());
    return Ok(Args::Direct {
        bot: bot_name(a.in_bot.unwrap()),
        packs: s,
    });
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<App, Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    let t = app.items.items[app.items.state.selected().unwrap()].pack as usize;
                    if app.selected.contains(&t) {
                        app.selected
                            .swap_remove(app.selected.iter().position(|x| *x == t).unwrap());
                    } else {
                        app.selected.push(t);
                    }
                    if app.in_bot.is_none() {
                        app.in_bot =
                            Some(app.items.items[app.items.state.selected().unwrap()].bot_id);
                        app.filter_items();
                    }
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    app.download = true;
                    break;
                }
                KeyCode::Esc | KeyCode::Char('q') => break,
                KeyCode::Down => app.items.next(),
                KeyCode::Up => app.items.previous(),
                _ => (),
            };
        }
    }
    Ok(app)
}
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let entry: Vec<ListItem> = iter_map!(app.items.items, |v| -> ListItem {
        let mut added = "";
        if app.selected.contains(&(v.pack as usize)) {
            added = "+ ";
        }
        ListItem::new(format!("{}{} :: {}", added, v.name, v.size)).style(Style::default())
    });
    let vids = List::new(entry)
        .block(Block::default().borders(Borders::ALL).title("packs"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(vids, f.size(), &mut app.items.state);
}
