use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, time::Duration};

// Import our custom modules
mod app;
mod ui;

use app::{App, AppTab, Phase};

fn main() -> Result<(), io::Error> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App Loop
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore Terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        // Draw the UI using the external ui module
        terminal.draw(|f| ui::ui(f, app))?;

        // Check for Auto-Complete and auto-transition to the next phase
        if app.running && app.get_remaining().is_zero() {
            app.next_phase();
        }

        // Handle Inputs
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                // Global Keys
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab => {
                        app.current_tab = match app.current_tab {
                            AppTab::Timer => AppTab::Settings,
                            AppTab::Settings => AppTab::Timer,
                        }
                    }
                    _ => {}
                }

                // Context Keys
                match app.current_tab {
                    AppTab::Timer => match key.code {
                        KeyCode::Char(' ') => app.toggle_timer(),
                        KeyCode::Char('n') => app.next_phase(),
                        KeyCode::Char('r') => app.reset_timer(),
                        KeyCode::Char('1') => {
                            app.phase = Phase::Focus;
                            app.reset_timer();
                        }
                        KeyCode::Char('2') => {
                            app.phase = Phase::ShortBreak;
                            app.reset_timer();
                        }
                        KeyCode::Char('3') => {
                            app.phase = Phase::LongBreak;
                            app.reset_timer();
                        }
                        _ => {}
                    },
                    AppTab::Settings => match key.code {
                        KeyCode::Up | KeyCode::Char('k') => app.prev_setting(),
                        KeyCode::Down | KeyCode::Char('j') => app.next_setting(),
                        KeyCode::Left | KeyCode::Char('h') => app.adjust_setting(-5),
                        KeyCode::Right | KeyCode::Char('l') => app.adjust_setting(5),
                        _ => {}
                    },
                }
            }
        }
    }
}
