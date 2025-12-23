use crate::client::Response;
use crate::http::{HttpFile, Request};

pub struct App {
    pub http_file: HttpFile,
    pub selected: usize,
    pub last_response: Option<Response>,
    pub should_quit: bool,
    pub focus: Focus,
    pub response_scroll: u16,
    pub loading: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    RequestList,
    ResponseBody,
}

impl App {
    pub fn new(http_file: HttpFile) -> Self {
        Self {
            http_file,
            selected: 0,
            last_response: None,
            should_quit: false,
            focus: Focus::RequestList,
            response_scroll: 0,
            loading: false,
        }
    }

    pub fn selected_request(&self) -> Option<&Request> {
        self.http_file.requests.get(self.selected)
    }

    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn select_next(&mut self) {
        if self.selected < self.http_file.requests.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::RequestList => Focus::ResponseBody,
            Focus::ResponseBody => Focus::RequestList,
        };
    }

    pub fn scroll_up(&mut self) {
        self.response_scroll = self.response_scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.response_scroll = self.response_scroll.saturating_add(1);
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
