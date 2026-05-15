use notify_rust::Notification;
use ratatui::style::Color;
use std::time::{Duration, Instant};
use rodio::{OutputStream, OutputStreamHandle, Sink, source::{SineWave, Source}};

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
    LongBreakInterval,
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

    // Audio
    pub _audio_stream: Option<OutputStream>,
    pub audio_handle: Option<OutputStreamHandle>,
}

impl App {
    pub fn new() -> Self {
        let (audio_stream, audio_handle) = match OutputStream::try_default() {
            Ok((stream, handle)) => (Some(stream), Some(handle)),
            Err(_) => (None, None),
        };

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

            _audio_stream: audio_stream,
            audio_handle,
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
        self.ring();
    }

    pub fn ring(&self) {
        if let Some(handle) = &self.audio_handle {
            if let Ok(sink) = Sink::try_new(handle) {
                // A simple "ring" sound: two short beeps
                let beep1 = SineWave::new(440.0)
                    .amplify(0.10)
                    .take_duration(Duration::from_millis(150));
                let silence = SineWave::new(440.0)
                    .amplify(0.0)
                    .take_duration(Duration::from_millis(50));
                let beep2 = SineWave::new(880.0)
                    .amplify(0.10)
                    .take_duration(Duration::from_millis(300));

                sink.append(beep1);
                sink.append(silence);
                sink.append(beep2);
                sink.detach(); // Let it play in the background
            }
        }
    }

    pub fn notify(&self, title: &str, body: &str) {
        let _ = Notification::new().summary(title).body(body).show();
    }

    // --- Configuration Logic ---

    pub fn next_setting(&mut self) {
        self.selected_setting = match self.selected_setting {
            SettingSelection::FocusTime => SettingSelection::ShortBreakTime,
            SettingSelection::ShortBreakTime => SettingSelection::LongBreakTime,
            SettingSelection::LongBreakTime => SettingSelection::LongBreakInterval,
            SettingSelection::LongBreakInterval => SettingSelection::FocusTime,
        };
    }

    pub fn prev_setting(&mut self) {
        self.selected_setting = match self.selected_setting {
            SettingSelection::FocusTime => SettingSelection::LongBreakInterval,
            SettingSelection::ShortBreakTime => SettingSelection::FocusTime,
            SettingSelection::LongBreakTime => SettingSelection::ShortBreakTime,
            SettingSelection::LongBreakInterval => SettingSelection::LongBreakTime,
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
            SettingSelection::LongBreakInterval => {
                // Use a smaller delta or handle differently?
                // The current main.rs passes ±5. For interval, maybe we just use signum if it's large?
                // Actually, let's just use the delta but normalize it to ±1 for the interval if we want,
                // or just let it be ±5. But usually people want 2, 3, 4, 5.
                // I'll adjust the logic to use delta.signum() if I want it to be 1 step,
                // but let's see how main.rs calls it.
                let step = if delta.abs() >= 5 { delta.signum() } else { delta };
                self.long_break_interval = (self.long_break_interval as i64 + step).max(1).min(10) as u8;
            }
        }
        self.reset_timer();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_break_interval_customization() {
        let mut app = App::new();
        
        // Default is 4
        assert_eq!(app.long_break_interval, 4);
        
        // Adjust to 2
        app.selected_setting = SettingSelection::LongBreakInterval;
        app.adjust_setting(-2);
        assert_eq!(app.long_break_interval, 2);
        
        // Transition to Long Break after 2 focus sessions
        app.phase = Phase::Focus;
        app.pomodoro_count = 1;
        app.next_phase(); // Completes 2nd focus session
        assert_eq!(app.phase, Phase::LongBreak);
        assert_eq!(app.pomodoro_count, 2);
    }

    #[test]
    fn test_long_break_interval_limits() {
        let mut app = App::new();
        app.selected_setting = SettingSelection::LongBreakInterval;
        
        // Default is 4. Each call with -5 adjusts by -1.
        app.adjust_setting(-5); // 3
        app.adjust_setting(-5); // 2
        app.adjust_setting(-5); // 1
        app.adjust_setting(-5); // stays 1 (min)
        assert_eq!(app.long_break_interval, 1);
        
        // Adjust up to max (10)
        for _ in 0..15 {
            app.adjust_setting(5);
        }
        assert_eq!(app.long_break_interval, 10);
    }
}
