mod application;
mod colorscheme;
mod drawing;
mod langs;
mod terminal_prep;
mod testkeys;

use std::panic;
use std::time::Duration;
use std::{fs::File, io::stdout};

use application::{App, Screen, TestState};
use colorscheme::Theme;
use crossterm::{execute, style::Print};
use drawing::draw;
use terminal_prep::{cleanup_terminal, init_terminal};
use testkeys::test_key_handle;

#[macro_use]
extern crate log;

use simplelog::*;

use crossterm::event::{poll, read, Event as CEvent};

use tui::{backend::CrosstermBackend, Terminal};

/// In case of panic restores terminal before program terminates
fn panic_hook(panic_info: &panic::PanicInfo) {
    cleanup_terminal();
    // from what I discovered 
    // overflows expects
    let msg = match panic_info.payload().downcast_ref::<String>() {
        Some(s) => format!("p! {}", s),
        // panic! macro, unwraps
        // dunno if its consisitent, doesn't matter though
        // from docs its commonly String or &'static str
        None => match panic_info.payload().downcast_ref::<&str>() {
            Some(s) => format!("oof! {}", s),
            None => String::from("weird panic hook"),
        },
    };



    let location = panic_info.location().unwrap();
    let mut sout = stdout();
    execute!(sout, Print(format!("{}\n{}\n", msg, location))).unwrap();
}

fn main() -> crossterm::Result<()> {
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("smokey.log").unwrap(),
    )
    .unwrap();
    init_terminal();
    panic::set_hook(Box::new(|info| panic_hook(info)));

    #[allow(unused_mut)]
    let mut sout = stdout();

    let backend = CrosstermBackend::new(sout);
    let mut terminal = Terminal::new(backend)?;

    let mut test = TestState::default();
    let mut app = App::create_app();
    let theme = Theme::initial();

    test.restart_test(&mut app, &theme);

    while !app.should_quit {

        match app.screen {
            Screen::Test => draw(&mut terminal, &mut app, &mut test),
            _ => todo!(),
        }

        if poll(Duration::from_millis(250))? {
            let read = read()?;
            if let CEvent::Key(event) = read {
                test_key_handle(event, &mut app, &mut test, &theme);
            }
        } else {
            // TODO a tick event?
            // sneak an afk?
            // Timeout expired and no `Event` is available
        }
    }

    cleanup_terminal();
    Ok(())
}
