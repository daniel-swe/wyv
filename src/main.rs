pub mod ui;
pub mod widgets;

use std::thread;
use std::time::Duration;

use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

use anyhow::Result;
use tui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut exit = false;
    while !exit {
        ui::draw(&mut terminal)?;

        if poll(Duration::from_secs(0))? {
            let event = read()?;

            match event {
                Event::Key(ke) => {
                    if ke.code == KeyCode::Char('c') && ke.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        exit = true;
                    }
                }
                Event::Mouse(_) => (),
                Event::Resize(_, _) => (),
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
