use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Tabs},
};
use tui_big_text::{BigText, PixelSize};

// Import types from our application logic module
use crate::app::{App, AppTab, SettingSelection};

// --- UI Rendering ---

pub fn ui(f: &mut Frame, app: &App) {
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
            "Controls: [Space] Toggle | [R] Reset | [N] Next Phase | [1/2/3] Set Phase | [Tab] Settings | [Q] Quit"
        }
        AppTab::Settings => {
            "Controls: [Up/Down] Select | [Left/Right] Adjust (±5m) | [Tab] Back to Timer"
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
            Constraint::Length(1), // Pomodoro Count
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
    let elapsed = app.get_elapsed().as_secs_f64();
    let ratio = (elapsed / total).clamp(0.0, 1.0);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Time Elapsed "),
        )
        .gauge_style(Style::default().fg(phase_color))
        .ratio(ratio)
        .label(format!("{:.0}%", ratio * 100.0));

    f.render_widget(gauge, gauge_layout[1]);

    // Pomodoro Count
    let count_text = Paragraph::new(format!(
        "Pomodoros Completed: {}/{}",
        app.pomodoro_count % app.long_break_interval,
        app.long_break_interval
    ))
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Center);
    f.render_widget(count_text, layout[7]);
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
