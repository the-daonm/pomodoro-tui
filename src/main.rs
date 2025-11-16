use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify_rust::Notification;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use std::{
    io,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Focus,
    Break,
}

impl Phase {
    fn name(&self) -> &'static str {
        match self {
            Phase::Focus => "FOCUS",
            Phase::Break => "BREAK",
        }
    }
}

struct App {
    phase: Phase,
    focus_minutes: u64,
    break_minutes: u64,
    running: bool,
    start_time: Instant,
    paused_time: Duration,
}

impl App {
    fn new() -> Self {
        Self {
            phase: Phase::Focus,
            focus_minutes: 25,
            break_minutes: 5,
            running: false,
            start_time: Instant::now(),
            paused_time: Duration::ZERO,
        }
    }

    fn phase_duration(&self) -> Duration {
        match self.phase {
            Phase::Focus => Duration::from_secs(self.focus_minutes * 60),
            Phase::Break => Duration::from_secs(self.break_minutes * 60),
        }
    }

    fn elapsed(&self) -> Duration {
        if self.running {
            self.paused_time + self.start_time.elapsed()
        } else {
            self.paused_time
        }
    }

    fn remaining(&self) -> Duration {
        let total = self.phase_duration();
        total.saturating_sub(self.elapsed())
    }

    fn toggle(&mut self) {
        if self.running {
            // pause
            self.paused_time = self.elapsed();
            self.running = false;
        } else {
            // resume
            self.start_time = Instant::now();
            self.running = true;
        }
    }

    fn next_phase(&mut self) {
        // switch phase
        self.phase = match self.phase {
            Phase::Focus => Phase::Break,
            Phase::Break => Phase::Focus,
        };

        // reset timer
        self.running = false;
        self.paused_time = Duration::ZERO;
        self.start_time = Instant::now();

        // notify user that next phase starts
        send_notification(
            &format!("Starting {}", self.phase.name()),
            "Pomodoro TUI started your next session.",
        );
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => app.toggle(),
                    KeyCode::Char('n') => app.next_phase(),
                    _ => {}
                }
            }
        }

        // if timer done, auto stop, send notification and wait for user to press "n"
        if app.remaining().as_secs() == 0 && app.running {
            app.toggle(); // auto pause

            send_notification(
                &format!("{} session finished", app.phase.name()),
                "Press N to go to the next phase.",
            );
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// small helper: fire a desktop notification; ignore errors if daemon missing
fn send_notification(title: &str, body: &str) {
    let _ = Notification::new().summary(title).body(body).show();
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    // Outer frame
    let outer = Block::default().borders(Borders::ALL).title(" Pomodoro ");
    let inner = outer.inner(size);

    // Draw outer block first
    f.render_widget(outer, size);

    // Split the *inside* of the block into rows
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2), // phase + status
                Constraint::Length(2), // time
                Constraint::Length(3), // progress bar
                Constraint::Min(0),    // spacer
                Constraint::Length(1), // controls
            ]
            .as_ref(),
        )
        .split(inner);

    let total = app.phase_duration().as_secs_f64().max(1.0);
    let remaining = app.remaining().as_secs_f64();
    let mut progress = 1.0 - (remaining / total);
    if !progress.is_finite() {
        progress = 0.0;
    }
    progress = progress.clamp(0.0, 1.0);

    let mins = app.remaining().as_secs() / 60;
    let secs = app.remaining().as_secs() % 60;

    // Phase + running/paused
    let status = if app.running { "RUNNING" } else { "PAUSED" };
    let phase_text = format!("Phase: {}  [{}]", app.phase.name(), status);
    let phase = Paragraph::new(phase_text).style(Style::default().fg(Color::Yellow));
    f.render_widget(phase, chunks[0]);

    // Time
    let time = Paragraph::new(format!("Time: {:02}:{:02}", mins, secs))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(time, chunks[1]);

    // Progress bar
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Progress "))
        .gauge_style(Style::default().fg(Color::Green))
        .label(format!("{:>3}%", (progress * 100.0).round() as u64))
        .ratio(progress);
    f.render_widget(gauge, chunks[2]);

    // Controls
    let help = Paragraph::new("Controls: [Space] Start/Stop  [N] Next Phase  [Q] Quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[4]);
}
