use crate::{argparse::Args, list::StatefulList};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
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
macro_rules! iter_collect {
    ($vector:expr,$func:expr) => {
        $vector.iter().map($func).collect()
    };
    (into $vector:expr,$func:expr) => {
        $vector.into_iter().map($func).collect()
    };
}

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
        SearchResultItem {
            bot_id: cap[1].parse::<u16>().unwrap(),
            pack: cap[2].parse::<u32>().unwrap(),
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

fn get_bots() -> HashMap<u16, String> {
    let a = get(format!("{}/bots", API_BASE)).unwrap().text().unwrap();
    let re_id = Regex::new(r#""id":(\d+),"name":"(.+?)""#).unwrap();
    let caps = re_id.captures_iter(&a);
    let mut map = HashMap::new();
    let _ = caps
        .into_iter()
        .map(|c| {
            map.insert(c[1].parse::<u16>().unwrap(), c[2].to_string());
        })
        .collect::<Vec<_>>();
    map
}

fn search_disp(query: &str) -> Result<Args, Box<dyn Error>> {
    let resp = get(format!("{}/search?query={}", API_BASE, query))
        .unwrap()
        .text()
        .unwrap();

    let search_re =
        Regex::new(r#""botId":(\d+),"number":(\d+),"name":"(.+?)","size":"(.+?)""#).unwrap();

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
    } else {
        let bot_name = |id: u16| {
            let map = get_bots();
            let name = map.get(&id).unwrap();
            name.to_string()
        };
        let a = res.unwrap();
        if a.download {
            let s: Vec<String> = a.selected.into_iter().map(|x| x.to_string()).collect();
            return Ok(Args::Direct {
                bot: bot_name(a.in_bot.unwrap()),
                packs: s,
            });
        } else {
            println!("nothing to download");
            std::process::exit(0);
        }
    }
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
    let entry: Vec<ListItem> = iter_collect!(app.items.items, |v| -> ListItem {
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
