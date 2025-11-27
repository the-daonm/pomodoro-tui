use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify_rust::Notification;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Tabs},
};
use std::{
    io,
    time::{Duration, Instant},
};
use tui_big_text::{BigText, PixelSize};

// --- Enums for State Management ---

#[derive(Clone, Copy, PartialEq, Debug)]
enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Phase {
    fn name(&self) -> &'static str {
        match self {
            Phase::Focus => "FOCUS SESSION",
            Phase::ShortBreak => "SHORT BREAK",
            Phase::LongBreak => "LONG BREAK",
        }
    }

    fn color(&self) -> Color {
        match self {
            Phase::Focus => Color::Red,
            Phase::ShortBreak => Color::Green,
            Phase::LongBreak => Color::Blue,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum AppTab {
    Timer,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum SettingSelection {
    FocusTime,
    ShortBreakTime,
    LongBreakTime,
}

// --- Main Application Struct ---

struct App {
    // Navigation
    current_tab: AppTab,

    // Timer State
    phase: Phase,
    running: bool,
    start_time: Instant,
    paused_duration: Duration, // Accumulated time passed before pause

    // Configuration (stored in minutes)
    cfg_focus: u64,
    cfg_short: u64,
    cfg_long: u64,

    // Settings Selection
    selected_setting: SettingSelection,
}

impl App {
    fn new() -> Self {
        Self {
            current_tab: AppTab::Timer,
            phase: Phase::Focus,
            running: false,
            start_time: Instant::now(),
            paused_duration: Duration::ZERO,
            cfg_focus: 25,
            cfg_short: 5,
            cfg_long: 15,
            selected_setting: SettingSelection::FocusTime,
        }
    }

    // --- Time Logic ---

    fn get_target_duration(&self) -> Duration {
        let mins = match self.phase {
            Phase::Focus => self.cfg_focus,
            Phase::ShortBreak => self.cfg_short,
            Phase::LongBreak => self.cfg_long,
        };
        Duration::from_secs(mins * 60)
    }

    fn get_elapsed(&self) -> Duration {
        if self.running {
            self.paused_duration + self.start_time.elapsed()
        } else {
            self.paused_duration
        }
    }

    fn get_remaining(&self) -> Duration {
        let target = self.get_target_duration();
        target.saturating_sub(self.get_elapsed())
    }

    fn toggle_timer(&mut self) {
        if self.running {
            // Pause
            self.paused_duration += self.start_time.elapsed();
            self.running = false;
        } else {
            // Resume
            self.start_time = Instant::now();
            self.running = true;
        }
    }

    fn reset_timer(&mut self) {
        self.running = false;
        self.paused_duration = Duration::ZERO;
        self.start_time = Instant::now();
    }

    fn next_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Focus => Phase::ShortBreak,
            Phase::ShortBreak => Phase::Focus, // Simple toggle for now, could be smarter
            Phase::LongBreak => Phase::Focus,
        };
        self.reset_timer();
        self.notify("Phase Changed", &format!("Starting {}", self.phase.name()));
    }

    fn notify(&self, title: &str, body: &str) {
        let _ = Notification::new().summary(title).body(body).show();
    }

    // --- Configuration Logic ---

    fn next_setting(&mut self) {
        self.selected_setting = match self.selected_setting {
            SettingSelection::FocusTime => SettingSelection::ShortBreakTime,
            SettingSelection::ShortBreakTime => SettingSelection::LongBreakTime,
            SettingSelection::LongBreakTime => SettingSelection::FocusTime,
        };
    }

    fn prev_setting(&mut self) {
        self.selected_setting = match self.selected_setting {
            SettingSelection::FocusTime => SettingSelection::LongBreakTime,
            SettingSelection::ShortBreakTime => SettingSelection::FocusTime,
            SettingSelection::LongBreakTime => SettingSelection::ShortBreakTime,
        };
    }

    fn adjust_setting(&mut self, delta: i64) {
        match self.selected_setting {
            SettingSelection::FocusTime => {
                self.cfg_focus = (self.cfg_focus as i64 + delta).max(1).min(120) as u64;
            }
            SettingSelection::ShortBreakTime => {
                self.cfg_short = (self.cfg_short as i64 + delta).max(1).min(60) as u64;
            }
            SettingSelection::LongBreakTime => {
                self.cfg_long = (self.cfg_long as i64 + delta).max(1).min(60) as u64;
            }
        }
        // If we adjust the time of the *current* phase, we should reset the timer to reflect it
        // otherwise math gets weird
        self.reset_timer();
    }
}

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
        terminal.draw(|f| ui(f, app))?;

        // Check for Auto-Complete
        if app.running && app.get_remaining().is_zero() {
            app.running = false;
            app.notify("Timer Finished!", "Time to switch phases.");
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

// --- UI Rendering ---

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();
    // Main Container
    let main_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    f.render_widget(main_block, size);

    // Layout: Tabs at top, Content in middle, Help at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .margin(1) // padding inside main border
        .split(size);

    // Tabs
    let titles = vec![" Timer ", " Settings "];
    let tab_style = match app.current_tab {
        AppTab::Timer => app.phase.color(),
        AppTab::Settings => Color::Cyan,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(match app.current_tab {
            AppTab::Timer => 0,
            _ => 1,
        })
        .highlight_style(Style::default().fg(tab_style).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    // Content
    match app.current_tab {
        AppTab::Timer => draw_timer_tab(f, app, chunks[1]),
        AppTab::Settings => draw_settings_tab(f, app, chunks[1]),
    };

    // Footer
    let footer_text = match app.current_tab {
        AppTab::Timer => {
            "Controls: [Space] Toggle | [R] Reset | [N] Next Phase | [Tab] Settings | [Q] Quit"
        }
        AppTab::Settings => {
            "Controls: [Up/Down] Select | [Left/Right] Adjust | [Tab] Back to Timer"
        }
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

fn draw_timer_tab(f: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // Top Spring
            Constraint::Length(1), // Phase Name (e.g., "FOCUS SESSION")
            Constraint::Length(1), // Status (e.g., "RUNNING")
            Constraint::Length(4), // Gap (Increased space)
            Constraint::Length(8), // Big Timer Height
            Constraint::Length(4), // Gap (Increased space)
            Constraint::Length(3), // Gauge Height
            Constraint::Fill(1),   // Bottom Spring
        ])
        .split(area);

    let phase_color = app.phase.color();

    // Phase Name
    let phase_text = Paragraph::new(app.phase.name())
        .style(
            Style::default()
                .fg(phase_color)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(phase_text, layout[1]);

    // Status
    let status_str = if app.running { "RUNNING" } else { "PAUSED" };
    let status_text = Paragraph::new(format!("[ {} ]", status_str))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(status_text, layout[2]);

    // Big Timer
    let remaining = app.get_remaining();
    let mins = remaining.as_secs() / 60;
    let secs = remaining.as_secs() % 60;
    let time_str = format!("{:02}:{:02}", mins, secs);

    let timer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),    // Spacer Left
            Constraint::Length(39), // Fixed width for "00:00"
            Constraint::Fill(1),    // Spacer Right
        ])
        .split(layout[4]);

    let big_text = BigText::builder()
        .pixel_size(PixelSize::Full)
        .style(Style::default().fg(if app.running {
            phase_color
        } else {
            Color::White
        }))
        .lines(vec![time_str.into()])
        .build();

    f.render_widget(big_text, timer_layout[1]);

    // Progress Gauge
    let gauge_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Percentage(80), // 80% width
            Constraint::Fill(1),
        ])
        .split(layout[6]);

    let total = app.get_target_duration().as_secs_f64();
    let current = remaining.as_secs_f64();
    let ratio = (current / total).clamp(0.0, 1.0);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Time Remaining "),
        )
        .gauge_style(Style::default().fg(phase_color))
        .ratio(ratio)
        .label(format!("{:.0}%", ratio * 100.0));

    f.render_widget(gauge, gauge_layout[1]);
}

fn draw_settings_tab(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Configuration ")
        .style(Style::default().fg(Color::Cyan));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Fill(1),
        ])
        .margin(2)
        .split(inner_area);

    // Helper to render a setting row
    let render_setting =
        |f: &mut Frame, label: &str, value: u64, selection: SettingSelection, index: usize| {
            let is_selected = app.selected_setting == selection;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let text = format!(" {}   < {:02} min > ", label, value);
            let p = Paragraph::new(text)
                .block(Block::default().borders(Borders::BOTTOM))
                .style(style)
                .alignment(Alignment::Center);
            f.render_widget(p, layout[index]);
        };

    render_setting(
        f,
        "Focus Duration",
        app.cfg_focus,
        SettingSelection::FocusTime,
        1,
    );
    render_setting(
        f,
        "Short Break Duration",
        app.cfg_short,
        SettingSelection::ShortBreakTime,
        2,
    );
    render_setting(
        f,
        "Long Break Duration",
        app.cfg_long,
        SettingSelection::LongBreakTime,
        3,
    );
}
