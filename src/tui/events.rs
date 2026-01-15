use super::app::{App, Focus, ResponseTab};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub enum EventResult {
    Continue,
    ExecuteRequest,
    ExecuteHistoryEntry,
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
        KeyCode::Char('H') if !app.filter_active => {
            app.toggle_history_view();
            return EventResult::Continue;
        }
        KeyCode::Esc if app.history_view_active => {
            app.toggle_history_view();
            return EventResult::Continue;
        }
        KeyCode::Tab => {
            if app.history_view_active {
                app.focus = match app.focus {
                    Focus::HistoryList => Focus::HistoryDetail,
                    Focus::HistoryDetail => Focus::HistoryList,
                    _ => Focus::HistoryList,
                };
            } else {
                app.toggle_focus();
            }
            return EventResult::Continue;
        }
        _ => {}
    }

    if app.history_view_active {
        match app.focus {
            Focus::HistoryList => handle_history_list_keys(app, key),
            Focus::HistoryDetail => handle_history_detail_keys(app, key),
            _ => EventResult::Continue,
        }
    } else {
        match app.focus {
            Focus::RequestList => handle_request_list_keys(app, key),
            Focus::ResponseBody => handle_response_keys(app, key),
            Focus::RequestDetails => handle_request_details_keys(app, key),
            Focus::VariablesList => handle_variables_keys(app, key),
            _ => EventResult::Continue,
        }
    }
}

fn handle_request_list_keys(app: &mut App, key: KeyEvent) -> EventResult {
    if app.filter_active {
        return handle_filter_keys(app, key);
    }

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
        KeyCode::Char('/') => {
            app.enter_filter_mode();
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}

fn handle_filter_keys(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Esc => {
            app.exit_filter_mode();
            EventResult::Continue
        }
        KeyCode::Backspace => {
            if app.filter_text.is_empty() {
                app.exit_filter_mode();
            } else {
                app.filter_backspace();
            }
            EventResult::Continue
        }
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.select_previous();
            EventResult::Continue
        }
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.select_next();
            EventResult::Continue
        }
        KeyCode::Up => {
            app.select_previous();
            EventResult::Continue
        }
        KeyCode::Down => {
            app.select_next();
            EventResult::Continue
        }
        KeyCode::Enter => EventResult::ExecuteRequest,
        KeyCode::Char(c) => {
            app.filter_append_char(c);
            EventResult::Continue
        }
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
        KeyCode::Char('h') | KeyCode::Right => {
            app.switch_to_headers_tab();
            EventResult::Continue
        }
        KeyCode::Char('b') | KeyCode::Left => {
            app.switch_to_body_tab();
            EventResult::Continue
        }
        KeyCode::Up | KeyCode::Char('k') => {
            match app.response_tab {
                ResponseTab::Body => app.scroll_up(),
                ResponseTab::Headers => app.scroll_headers_up(),
            }
            EventResult::Continue
        }
        KeyCode::Down | KeyCode::Char('j') => {
            match app.response_tab {
                ResponseTab::Body => app.scroll_down(),
                ResponseTab::Headers => app.scroll_headers_down(),
            }
            EventResult::Continue
        }
        KeyCode::PageUp => {
            for _ in 0..10 {
                match app.response_tab {
                    ResponseTab::Body => app.scroll_up(),
                    ResponseTab::Headers => app.scroll_headers_up(),
                }
            }
            EventResult::Continue
        }
        KeyCode::PageDown => {
            for _ in 0..10 {
                match app.response_tab {
                    ResponseTab::Body => app.scroll_down(),
                    ResponseTab::Headers => app.scroll_headers_down(),
                }
            }
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}

fn handle_request_details_keys(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.scroll_request_details_up();
            EventResult::Continue
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.scroll_request_details_down();
            EventResult::Continue
        }
        KeyCode::PageUp => {
            for _ in 0..10 {
                app.scroll_request_details_up();
            }
            EventResult::Continue
        }
        KeyCode::PageDown => {
            for _ in 0..10 {
                app.scroll_request_details_down();
            }
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}

fn handle_history_list_keys(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous_history();
            EventResult::Continue
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next_history();
            EventResult::Continue
        }
        KeyCode::Enter => EventResult::ExecuteHistoryEntry,
        _ => EventResult::Continue,
    }
}

fn handle_history_detail_keys(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.scroll_history_detail_up();
            EventResult::Continue
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.scroll_history_detail_down();
            EventResult::Continue
        }
        KeyCode::PageUp => {
            for _ in 0..10 {
                app.scroll_history_detail_up();
            }
            EventResult::Continue
        }
        KeyCode::PageDown => {
            for _ in 0..10 {
                app.scroll_history_detail_down();
            }
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}
