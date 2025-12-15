# 🍅 TUI Pomodoro Timer

> A minimalist, customizable, and scalable Pomodoro timer built in Rust using the ratatui (formerly tui-rs) and crossterm libraries.

## ✨ Features

- Customizable Sessions: Easily configure Focus, Short Break, and Long Break durations via a dedicated settings tab.
- True Pomodoro Logic: Automatically transitions phases, including a configurable Long Break after a set number of Focus sessions (default 4).
- Desktop Notifications: Uses notify-rust to send system notifications when a phase ends, allowing you to focus without staring at the terminal.
- Scalable Architecture: Code is organized into three distinct modules (main.rs, app.rs, ui.rs) for clean separation of concerns and easy maintenance.
- Responsive UI: Built with ratatui for a clean, modern Terminal User Interface.

## 📦 Installation
### Prerequisites

You need the Rust toolchain installed. If you don't have it, you can install it via rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and Run

- Clone the repository:
```bash
git clone https://github.com/the-daonm/pomodoro-tui.git
cd pomodoro-tui
```

- Run the application:
```bash
cargo run
```

Note: On Linux, ensure you have a notification server installed (like dunst or gnome-shell) for phase notifications to work.

## 🕹️ Usage & Controls

The application uses simple keyboard shortcuts to manage the timer and settings.
| Key           | Context | Action                                               |
|---------------|---------|------------------------------------------------------|
| Space         | Timer   | Toggle (Start/Pause) the current session.            |
| R             | Timer   | Reset the current timer and return to initial time.  |
| N             | Timer   | Skip to the next phase (triggers full Pomodoro cycle logic). |
| 1/2/3         | Timer   | Immediately set phase to Focus (1), Short Break (2), or Long Break (3). |
| Tab           | Global  | Switch between Timer and Settings tabs.              |
| Up/Down (K/J) | Settings| Select the configuration setting to change.          |
| Left/Right (H/L)|Settings| Adjust the selected duration (default adjustment is ±5 minutes). |
| Q             | Global  | Quit the application.                                |
## ⚙️ Project Structure

The project employs a modular structure to keep logic and rendering decoupled, which is highly recommended for ratatui applications.
| File          | Role                  | Description                                                |
|---------------|-----------------------|------------------------------------------------------------|
| src/main.rs   | Entry Point/Event Loop| Handles TUI setup/teardown (crossterm) and the main run_app loop, including input event processing and phase auto-transition. |
| src/app.rs    | Application Logic     | Defines the central App state struct, phase enums, timer calculations, Pomodoro cycle logic, and configuration adjustment methods. |
| src/ui.rs     | Rendering             | Contains the top-level ui function and all detailed functions for drawing the Timer and Settings tabs (ratatui, tui-big-text widgets). |
## 🛠️ Customization (Configuration)

From the Settings tab, you can customize the following durations (in minutes):
| Setting            | Default Value | Description                                                    |
|--------------------|---------------|----------------------------------------------------------------|
| Focus Duration     | 25            | Length of the work/focus session.                              |
| Short Break Duration| 5             | Length of the short rest period.                               |
| Long Break Duration| 15            | Length of the extended rest period (after 4 focus cycles).     |
## 🤝 Contributing & Future Plans

This project is ready for growth! Feel free to contribute by opening issues or submitting pull requests.
### Next Steps for Development:

- Persistent Configuration: Implement saving and loading settings (Focus, Breaks, Long Break Interval) to a file (e.g., JSON or TOML).

- Time Logging: Add a simple log file to track completed focus sessions and total time worked.

- More User Feedback: Add a visual indicator (like a small checkmark) in the Timer tab to show when a Pomodoro cycle is complete.

- Error Handling: Improve robustness with more graceful handling of I/O errors.
