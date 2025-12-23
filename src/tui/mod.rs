pub mod app;
pub mod events;
pub mod ui;

use crate::client::Client;
use crate::http::HttpFile;
pub use app::App;
use crossterm::{
    event::Event,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
pub use events::{EventResult, handle_key_event, poll_event};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};
use std::time::Duration;
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
        terminal.draw(|frame| render(frame, &app))?;

        if let Some(event) = poll_event(Duration::from_millis(100))?
            && let Event::Key(key) = event
        {
            match handle_key_event(&mut app, key) {
                EventResult::Quit => break,
                EventResult::ExecuteRequest => {
                    if let Some(request) = app.selected_request().cloned() {
                        app.loading = true;
                        app.response_scroll = 0;

                        terminal.draw(|frame| render(frame, &app))?;

                        match client.execute(&request, &app.http_file.variables).await {
                            Ok(response) => {
                                app.last_response = Some(response);
                            }
                            Err(e) => {
                                app.last_response = Some(crate::client::Response {
                                    status: 0,
                                    status_text: "Error".to_string(),
                                    headers: vec![],
                                    body: format!("Error: {}", e),
                                    duration: Duration::ZERO,
                                });
                            }
                        }
                        app.loading = false;
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
