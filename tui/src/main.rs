mod color;
mod app;
mod ui;

use std::{error::Error, io};
use app::App;
use ui::ui;
use openai;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?; // Enable mouse capture
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut text_area = tui_textarea::TextArea::default();
    // Create app and run it
    let mut app = App::new(&mut text_area);
    app.messages.push(openai::simple_message("hello".to_owned(), openai::Role::User));
    let res = run_app(&mut terminal, &mut app);

    if let Err(e) = res {
        eprintln!("Error running app: {}", e);
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture, // Disable mouse capture on exit
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('c') => return Ok(()), // Exit on Ctrl+C
                    KeyCode::Char('n') => {
                        // Create a new session
                        app.title = "New Codr Session".to_string();
                        app.mode = app::AppMode::Normal;
                        app.is_processing = false;
                    }
                    _ => {}
                }
            }  

            match app.mode {
                app::AppMode::Normal => {
                    if key.code == KeyCode::Enter {
                        if key.modifiers.contains(KeyModifiers::ALT) {
                            app.input.insert_newline();
                            continue;
                        } else {
                            app.is_processing = true;
                            app.mode = app::AppMode::Processing;
                            continue
                        }
                    }
                    app.input.input(key);
                },
                _ => {}
            }
        } else {
            // Ignore non-key events (e.g., mouse events) to prevent interference
            continue;
        }
    }
}

