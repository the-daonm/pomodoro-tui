use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Tabs, BorderType},
};
use tui_big_text::{BigText, PixelSize};

// Import types from our application logic module
use crate::app::{App, AppTab, SettingSelection};

// --- UI Rendering ---

pub fn ui(f: &mut Frame, app: &App) {
    let size = f.area();
    let phase_color = app.phase.color();

    // Main Container with dynamic border color
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(phase_color))
        .title(" POMODORO TUI ")
        .title_alignment(Alignment::Center);
    f.render_widget(main_block, size);

    // Layout: Tabs at top, Content in middle, Help at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .margin(1)
        .split(size);

    // Tabs
    let titles = vec![" [1] Timer ", " [2] Settings "];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)))
        .select(match app.current_tab {
            AppTab::Timer => 0,
            _ => 1,
        })
        .highlight_style(Style::default().fg(phase_color).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    // Content
    match app.current_tab {
        AppTab::Timer => draw_timer_tab(f, app, chunks[1]),
        AppTab::Settings => draw_settings_tab(f, app, chunks[1]),
    };

    // Footer
    let footer_text = if size.width < 70 {
        match app.current_tab {
            AppTab::Timer => " [Spc] Play/P | [N] Next | [R] Reset | [Tab] Tab | [Q] Q ",
            AppTab::Settings => " [↑/↓] Nav | [←/→] Adj | [Tab] Tab | [Q] Q ",
        }
    } else {
        match app.current_tab {
            AppTab::Timer => {
                " [Space] Play/Pause | [N] Next | [R] Reset | [Tab] Switch Tab | [Q] Quit "
            }
            AppTab::Settings => {
                " [↑/↓] Navigate | [←/→] Adjust | [Tab] Switch Tab | [Q] Quit "
            }
        }
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

fn draw_timer_tab(f: &mut Frame, app: &App, area: Rect) {
    // Adaptive Layout: Switch to Vertical if terminal is narrow
    if area.width < 80 {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(area);

        draw_main_timer(f, app, main_layout[0]);
        // Only draw stats if there's enough height
        if main_layout[1].height > 5 {
            draw_stats_sidebar(f, app, main_layout[1]);
        }
    } else {
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(area);

        draw_main_timer(f, app, main_layout[0]);
        draw_stats_sidebar(f, app, main_layout[1]);
    }
}

fn draw_main_timer(f: &mut Frame, app: &App, area: Rect) {
    let phase_color = app.phase.color();

    // Determine if we should use BigText and which size
    // Full size BigText "00:00" needs ~40 width and ~8 height
    // Quadrant size BigText "00:00" needs ~20 width and ~4 height
    let (use_big_text, pixel_size, timer_height) = if area.width >= 44 && area.height >= 12 {
        (true, PixelSize::Full, 8)
    } else if area.width >= 22 && area.height >= 8 {
        (true, PixelSize::Quadrant, 4)
    } else {
        (false, PixelSize::Quadrant, 1)
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // Spacer
            Constraint::Length(1), // Phase
            Constraint::Length(1), // Status
            Constraint::Length(timer_height), // Timer
            Constraint::Length(if area.height > 10 { 3 } else { 1 }), // Gauge
            Constraint::Length(if area.height > 12 { 2 } else { 0 }), // Pomodoro Icons
            Constraint::Fill(1),   // Spacer
        ])
        .margin(if area.width > 40 { 1 } else { 0 })
        .split(area);

    // 1. Phase Name
    let phase_text = Paragraph::new(app.phase.name())
        .style(Style::default().fg(phase_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(phase_text, layout[1]);

    // 2. Status
    let status_str = if app.running { "▶ RUNNING" } else { "⏸ PAUSED" };
    let status_color = if app.running { Color::LightGreen } else { Color::LightYellow };
    let status_text = Paragraph::new(status_str)
        .style(Style::default().fg(status_color))
        .alignment(Alignment::Center);
    f.render_widget(status_text, layout[2]);

    // 3. Timer
    let remaining = app.get_remaining();
    let mins = remaining.as_secs() / 60;
    let secs = remaining.as_secs() % 60;
    let time_str = format!("{:02}:{:02}", mins, secs);

    if use_big_text {
        let big_text_width = if pixel_size == PixelSize::Full { 40 } else { 20 };
        let timer_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(big_text_width),
                Constraint::Fill(1),
            ])
            .split(layout[3])[1];

        let big_text = BigText::builder()
            .pixel_size(pixel_size)
            .style(Style::default().fg(if app.running { phase_color } else { Color::White }))
            .lines(vec![time_str.into()])
            .build();
        f.render_widget(big_text, timer_area);
    } else {
        let timer_text = Paragraph::new(time_str)
            .style(Style::default().fg(if app.running { phase_color } else { Color::White }).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(timer_text, layout[3]);
    }

    // 4. Progress Gauge
    if layout[4].height > 0 {
        let total = app.get_target_duration().as_secs_f64();
        let elapsed = app.get_elapsed().as_secs_f64();
        let ratio = (elapsed / total).clamp(0.0, 1.0);

        let label = if layout[4].width > 10 {
            format!("{:.0}%", ratio * 100.0)
        } else {
            "".to_string()
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(phase_color).bg(Color::DarkGray))
            .ratio(ratio)
            .label(label)
            .use_unicode(true);
        f.render_widget(gauge, layout[4]);
    }

    // 5. Pomodoro Icons
    if layout[5].height > 0 {
        let mut spans = Vec::new();
        let current_session_in_cycle = app.pomodoro_count % app.long_break_interval;
        
        for i in 0..app.long_break_interval {
            if i < current_session_in_cycle {
                spans.push(Span::styled(" ● ", Style::default().fg(phase_color)));
            } else {
                spans.push(Span::styled(" ○ ", Style::default().fg(Color::DarkGray)));
            }
        }
        
        let icons = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        f.render_widget(icons, layout[5]);
    }
}

fn draw_stats_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let stats_block_height = if area.height > 6 { 6 } else { 4 };
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(stats_block_height), // Stats Block
            Constraint::Min(0),    // History Block
        ])
        .split(area);

    // Stats Block
    let total_mins = app.total_focus_seconds / 60;
    let mut stats_text = vec![
        Line::from(vec![
            Span::raw(" Focus: "),
            Span::styled(format!("{}m", total_mins), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw(" Done:  "),
            Span::styled(format!("{}", app.pomodoro_count), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
    ];

    if stats_block_height > 4 {
        stats_text.push(Line::from(vec![
            Span::raw(" Cycle: "),
            Span::styled(format!("{}/{}", app.pomodoro_count % app.long_break_interval, app.long_break_interval), Style::default().fg(Color::Magenta)),
        ]));
    }

    let stats_block = Paragraph::new(stats_text)
        .block(Block::default().title(" Stats ").borders(Borders::ALL).border_type(BorderType::Plain).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(stats_block, chunks[0]);

    // History Block - Only if there is room
    if chunks[1].height > 3 {
        let history_items: Vec<ListItem> = app.history.iter().rev().map(|(name, duration)| {
            let color = if name.contains("FOCUS") { Color::LightRed } else if name.contains("SHORT") { Color::LightGreen } else { Color::LightBlue };
            let label = if chunks[1].width > 15 {
                format!(" • {:<10}", name)
            } else {
                format!("•{:<1}", &name[..1])
            };
            
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(label, Style::default().fg(color)),
                    Span::styled(format!(" {:>2}m", duration), Style::default().fg(Color::DarkGray)),
                ])
            ])
        }).collect();

        let history_list = List::new(history_items)
            .block(Block::default().title(" Recent ").borders(Borders::ALL).border_type(BorderType::Plain).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(history_list, chunks[1]);
    }
}

fn draw_settings_tab(f: &mut Frame, app: &App, area: Rect) {
    let desc_height = if area.height > 10 { 3 } else if area.height > 6 { 1 } else { 0 };
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(desc_height), // Description
            Constraint::Min(0),    // Settings List
        ])
        .margin(if area.width > 40 { 2 } else { 0 })
        .split(area);

    // Description area
    if desc_height > 0 {
        let desc = match app.selected_setting {
            SettingSelection::FocusTime => "Duration of your deep work sessions (typically 25m).",
            SettingSelection::ShortBreakTime => "Short rest after a focus session (typically 5m).",
            SettingSelection::LongBreakTime => "Longer rest after completing a cycle (typically 15-30m).",
            SettingSelection::LongBreakInterval => "Number of focus sessions before a long break.",
        };
        let desc_p = Paragraph::new(desc)
            .block(if desc_height > 1 { Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)) } else { Block::default() })
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC));
        f.render_widget(desc_p, chunks[0]);
    }

    // Settings List
    let label_width = if area.width > 50 { 25 } else { 15 };
    let items = vec![
        render_setting_item("Focus Duration", format!("{} min", app.cfg_focus), app.selected_setting == SettingSelection::FocusTime, label_width),
        render_setting_item("Short Break Duration", format!("{} min", app.cfg_short), app.selected_setting == SettingSelection::ShortBreakTime, label_width),
        render_setting_item("Long Break Duration", format!("{} min", app.cfg_long), app.selected_setting == SettingSelection::LongBreakTime, label_width),
        render_setting_item("Long Break Interval", format!("{} sessions", app.long_break_interval), app.selected_setting == SettingSelection::LongBreakInterval, label_width),
    ];

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default().bg(Color::DarkGray));
    
    f.render_widget(list, chunks[1]);
}

fn render_setting_item(label: &str, value: String, is_selected: bool, label_width: usize) -> ListItem<'_> {
    let (prefix, style) = if is_selected {
        (" │ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        ("   ", Style::default().fg(Color::Reset))
    };

    ListItem::new(vec![
        Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:<width$}", label, width = label_width), style),
            Span::styled(format!(" [ {} ] ", value), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""), // Spacing
    ])
}
