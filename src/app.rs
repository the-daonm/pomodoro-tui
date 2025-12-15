use notify_rust::Notification;
use ratatui::style::Color;
use std::time::{Duration, Instant};

// --- Enums for State Management ---

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn name(&self) -> &'static str {
        match self {
            Phase::Focus => "FOCUS SESSION",
            Phase::ShortBreak => "SHORT BREAK",
            Phase::LongBreak => "LONG BREAK",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Phase::Focus => Color::Red,
            Phase::ShortBreak => Color::Green,
            Phase::LongBreak => Color::Blue,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AppTab {
    Timer,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SettingSelection {
    FocusTime,
    ShortBreakTime,
    LongBreakTime,
}

// --- Main Application Struct ---

pub struct App {
    // Navigation
    pub current_tab: AppTab,

    // Timer State
    pub phase: Phase,
    pub running: bool,
    pub start_time: Instant,
    pub paused_duration: Duration, // Accumulated time passed before pause

    // Pomodoro Logic
    pub pomodoro_count: u8, // Tracks completed focus sessions (0 to 3 before Long Break)
    pub long_break_interval: u8, // Define the interval for a long break (e.g., 4 sessions)

    // Configuration (stored in minutes)
    pub cfg_focus: u64,
    pub cfg_short: u64,
    pub cfg_long: u64,

    // Settings Selection
    pub selected_setting: SettingSelection,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_tab: AppTab::Timer,
            phase: Phase::Focus,
            running: false,
            start_time: Instant::now(),
            paused_duration: Duration::ZERO,

            pomodoro_count: 0,
            long_break_interval: 4,

            cfg_focus: 25,
            cfg_short: 5,
            cfg_long: 15,
            selected_setting: SettingSelection::FocusTime,
        }
    }

    // --- Time Logic ---

    pub fn get_target_duration(&self) -> Duration {
        let mins = match self.phase {
            Phase::Focus => self.cfg_focus,
            Phase::ShortBreak => self.cfg_short,
            Phase::LongBreak => self.cfg_long,
        };
        Duration::from_secs(mins * 60)
    }

    pub fn get_elapsed(&self) -> Duration {
        if self.running {
            self.paused_duration + self.start_time.elapsed()
        } else {
            self.paused_duration
        }
    }

    pub fn get_remaining(&self) -> Duration {
        let target = self.get_target_duration();
        target.saturating_sub(self.get_elapsed())
    }

    pub fn toggle_timer(&mut self) {
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

    pub fn reset_timer(&mut self) {
        self.running = false;
        self.paused_duration = Duration::ZERO;
        self.start_time = Instant::now();
    }

    /// Core Pomodoro logic: Handles phase transition and updates the pomodoro count.
    pub fn next_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Focus => {
                self.pomodoro_count += 1;
                if self.pomodoro_count % self.long_break_interval == 0 {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            // Breaks always transition back to a Focus session
            Phase::ShortBreak | Phase::LongBreak => Phase::Focus,
        };
        self.reset_timer();
        self.notify("Phase Changed", &format!("Starting {}", self.phase.name()));
    }

    pub fn notify(&self, title: &str, body: &str) {
        let _ = Notification::new().summary(title).body(body).show();
    }

    // --- Configuration Logic ---

    pub fn next_setting(&mut self) {
        self.selected_setting = match self.selected_setting {
            SettingSelection::FocusTime => SettingSelection::ShortBreakTime,
            SettingSelection::ShortBreakTime => SettingSelection::LongBreakTime,
            SettingSelection::LongBreakTime => SettingSelection::FocusTime,
        };
    }

    pub fn prev_setting(&mut self) {
        self.selected_setting = match self.selected_setting {
            SettingSelection::FocusTime => SettingSelection::LongBreakTime,
            SettingSelection::ShortBreakTime => SettingSelection::FocusTime,
            SettingSelection::LongBreakTime => SettingSelection::ShortBreakTime,
        };
    }

    pub fn adjust_setting(&mut self, delta: i64) {
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
        self.reset_timer();
    }
}
