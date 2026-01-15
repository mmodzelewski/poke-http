pub mod app;
pub mod events;
pub mod ui;

use crate::client::Client;
use crate::http::{HttpFile, Request};
pub use app::{App, HistoryEntry};
use crossterm::{
    event::Event,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
pub use events::{EventResult, handle_key_event, poll_event};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};
use std::time::{Duration, SystemTime};
pub use ui::render;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

pub async fn run(http_file: HttpFile) -> anyhow::Result<()> {
    let mut terminal = init_terminal()?;
    let mut app = App::new(http_file);
    let client = Client::new();

    loop {
        terminal.draw(|frame| render(frame, &mut app))?;

        if let Some(event) = poll_event(Duration::from_millis(100))?
            && let Event::Key(key) = event
        {
            match handle_key_event(&mut app, key) {
                EventResult::Quit => break,
                EventResult::ExecuteRequest => {
                    if let Some(request) = app.selected_request().cloned() {
                        execute_request(&mut app, &client, &mut terminal, request).await;
                    }
                }
                EventResult::ExecuteHistoryEntry => {
                    if let Some(entry) = app.selected_history_entry() {
                        let request = entry.request.clone();
                        execute_request(&mut app, &client, &mut terminal, request).await;
                    }
                }
                EventResult::Continue => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}

async fn execute_request(app: &mut App, client: &Client, terminal: &mut Tui, request: Request) {
    app.loading = true;
    app.response_scroll = 0;

    terminal.draw(|frame| render(frame, app)).ok();

    let timestamp = SystemTime::now();

    match client.execute(&request, &app.http_file.variables).await {
        Ok(response) => {
            let history_entry = HistoryEntry {
                request: request.clone(),
                response: response.clone(),
                timestamp,
            };
            app.add_history_entry(history_entry);
            app.last_response = Some(response);
        }
        Err(e) => {
            let error_response = crate::client::Response {
                status: 0,
                status_text: "Error".to_string(),
                headers: vec![],
                body: format!("Error: {}", e),
                duration: Duration::ZERO,
            };

            let history_entry = HistoryEntry {
                request: request.clone(),
                response: error_response.clone(),
                timestamp,
            };
            app.add_history_entry(history_entry);
            app.last_response = Some(error_response);
        }
    }

    app.loading = false;

    if app.history_view_active {
        app.selected_history = app.history.len().saturating_sub(1);
    }
}
