//! repotui - A TUI tool for browsing and managing GitHub repositories
//!
//! # Architecture
//!
//! This application follows the Elm architecture pattern:
//! - **Model**: Application state (`app::model::App`)
//! - **Msg**: Messages that trigger updates (`app::msg::AppMsg`)
//! - **Update**: State transitions (`app::update::update`)
//! - **View**: UI rendering (`ui::render::render`)
//!
//! # Example
//!
//! ```rust,no_run
//! use repotui::run;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     run().await
//! }
//! ```

pub mod action;
pub mod app;
pub mod config;
pub mod error;
pub mod git;
pub mod handler;
pub mod repo;
pub mod runtime;
pub mod ui;

mod constants;

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize logging
fn init_logging() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    Ok(())
}

/// Initialize terminal
fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

/// Restore terminal
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// Run the application
pub async fn run() -> Result<()> {
    init_logging()?;
    tracing::info!("Starting repotui");

    let mut terminal = init_terminal()?;
    terminal.clear()?;

    let result = run_app(&mut terminal).await;

    restore_terminal()?;

    result
}

/// Main application loop
async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    use app::model::App;
    use runtime::executor::Runtime;
    use tokio::sync::mpsc;

    let (msg_tx, mut msg_rx) = mpsc::channel::<app::msg::AppMsg>(100);
    let runtime = Runtime::new(msg_tx.clone());

    // Load configuration
    runtime.dispatch(app::msg::Cmd::LoadConfig);

    let mut app = App::new(msg_tx);

    loop {
        // Render
        terminal.draw(|frame| ui::render::render(frame, &mut app))?;

        // Handle messages
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    handler::handle_key_event(key, &mut app, &runtime);
                }
                Event::Mouse(mouse) => {
                    if let Some(msg) = handler::handle_mouse_event(mouse, &app) {
                        app::update::update(msg, &mut app, &runtime);
                    }
                }
                _ => {}
            }
        }

        // Receive async messages
        if let Ok(msg) = msg_rx.try_recv() {
            app::update::update(msg, &mut app, &runtime);
        }

        // Check for quit
        if app.state == app::state::AppState::Quit {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging() {
        assert!(init_logging().is_ok());
    }
}
