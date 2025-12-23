use super::app::{App, Focus};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub enum EventResult {
    Continue,
    ExecuteRequest,
    Quit,
}

pub fn poll_event(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

pub fn handle_key_event(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('q') => return EventResult::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return EventResult::Quit;
        }
        KeyCode::Tab => {
            app.toggle_focus();
            return EventResult::Continue;
        }
        _ => {}
    }

    match app.focus {
        Focus::RequestList => handle_request_list_keys(app, key),
        Focus::VariablesList => handle_variables_keys(app, key),
        Focus::ResponseBody => handle_response_keys(app, key),
    }
}

fn handle_request_list_keys(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
            EventResult::Continue
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
            EventResult::Continue
        }
        KeyCode::Enter => EventResult::ExecuteRequest,
        _ => EventResult::Continue,
    }
}

fn handle_variables_keys(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous_variable();
            EventResult::Continue
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next_variable();
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}

fn handle_response_keys(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.scroll_up();
            EventResult::Continue
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.scroll_down();
            EventResult::Continue
        }
        KeyCode::PageUp => {
            for _ in 0..10 {
                app.scroll_up();
            }
            EventResult::Continue
        }
        KeyCode::PageDown => {
            for _ in 0..10 {
                app.scroll_down();
            }
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}
