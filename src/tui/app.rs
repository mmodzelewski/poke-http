use crate::client::Response;
use crate::http::{HttpFile, Request};
use regex::Regex;
use std::collections::HashSet;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub request: Request,
    pub response: Response,
    pub timestamp: SystemTime,
}

pub struct App {
    pub http_file: HttpFile,
    pub selected: usize,
    pub selected_variable: usize,
    pub last_response: Option<Response>,
    pub should_quit: bool,
    pub focus: Focus,
    pub response_scroll: u16,
    pub response_tab: ResponseTab,
    pub headers_scroll: u16,
    pub request_details_scroll: u16,
    pub request_details_visible_height: u16,
    pub loading: bool,
    pub filter_text: String,
    pub filter_active: bool,
    pub history: Vec<HistoryEntry>,
    pub history_view_active: bool,
    pub selected_history: usize,
    pub history_detail_scroll: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    RequestList,
    ResponseBody,
    RequestDetails,
    VariablesList,
    HistoryList,
    HistoryDetail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResponseTab {
    #[default]
    Body,
    Headers,
}

impl App {
    pub fn new(http_file: HttpFile) -> Self {
        Self {
            http_file,
            selected: 0,
            selected_variable: 0,
            last_response: None,
            should_quit: false,
            focus: Focus::RequestList,
            response_scroll: 0,
            response_tab: ResponseTab::default(),
            headers_scroll: 0,
            request_details_scroll: 0,
            request_details_visible_height: 0,
            loading: false,
            filter_text: String::new(),
            filter_active: false,
            history: Vec::new(),
            history_view_active: false,
            selected_history: 0,
            history_detail_scroll: 0,
        }
    }

    pub fn selected_request(&self) -> Option<&Request> {
        if self.filter_active {
            self.filtered_requests()
                .get(self.selected)
                .map(|(idx, _)| &self.http_file.requests[*idx])
        } else {
            self.http_file.requests.get(self.selected)
        }
    }

    pub fn filtered_requests(&self) -> Vec<(usize, &Request)> {
        if !self.filter_active || self.filter_text.is_empty() {
            return self.http_file.requests.iter().enumerate().collect();
        }

        let filter_lower = self.filter_text.to_lowercase();
        self.http_file
            .requests
            .iter()
            .enumerate()
            .filter(|(_, req)| {
                let name_matches = req
                    .name
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&filter_lower))
                    .unwrap_or(false);
                let url_matches = req.url.to_lowercase().contains(&filter_lower);
                name_matches || url_matches
            })
            .collect()
    }

    pub fn enter_filter_mode(&mut self) {
        self.filter_active = true;
        self.selected = 0;
    }

    pub fn exit_filter_mode(&mut self) {
        self.filter_active = false;
        self.filter_text.clear();
        self.selected = 0;
    }

    pub fn filter_append_char(&mut self, c: char) {
        self.filter_text.push(c);
        self.selected = 0;
    }

    pub fn filter_backspace(&mut self) {
        self.filter_text.pop();
        self.selected = 0;
    }

    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.selected_variable = 0;
            self.request_details_scroll = 0;
        }
    }

    pub fn select_next(&mut self) {
        let max = if self.filter_active {
            self.filtered_requests().len().saturating_sub(1)
        } else {
            self.http_file.requests.len().saturating_sub(1)
        };
        if self.selected < max {
            self.selected += 1;
            self.selected_variable = 0;
            self.request_details_scroll = 0;
        }
    }

    pub fn select_previous_variable(&mut self) {
        if self.selected_variable > 0 {
            self.selected_variable -= 1;
        }
    }

    pub fn select_next_variable(&mut self) {
        let max = self.get_used_variables().len().saturating_sub(1);
        if self.selected_variable < max {
            self.selected_variable += 1;
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::RequestList => Focus::ResponseBody,
            Focus::ResponseBody => Focus::RequestDetails,
            Focus::RequestDetails => Focus::VariablesList,
            Focus::VariablesList => Focus::RequestList,
            Focus::HistoryList | Focus::HistoryDetail => Focus::HistoryList,
        };
    }

    pub fn scroll_up(&mut self) {
        self.response_scroll = self.response_scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.response_scroll = self.response_scroll.saturating_add(1);
    }

    pub fn switch_to_body_tab(&mut self) {
        self.response_tab = ResponseTab::Body;
    }

    pub fn switch_to_headers_tab(&mut self) {
        self.response_tab = ResponseTab::Headers;
    }

    pub fn scroll_headers_up(&mut self) {
        self.headers_scroll = self.headers_scroll.saturating_sub(1);
    }

    pub fn scroll_headers_down(&mut self) {
        self.headers_scroll = self.headers_scroll.saturating_add(1);
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn scroll_request_details_up(&mut self) {
        self.request_details_scroll = self.request_details_scroll.saturating_sub(1);
    }

    pub fn scroll_request_details_down(&mut self) {
        let content_lines = self.request_details_content_lines() as u16;
        let visible = self.request_details_visible_height.saturating_sub(2);
        let max_scroll = content_lines.saturating_sub(visible);
        if self.request_details_scroll < max_scroll {
            self.request_details_scroll = self.request_details_scroll.saturating_add(1);
        }
    }

    fn request_details_content_lines(&self) -> usize {
        let Some(request) = self.selected_request() else {
            return 1;
        };

        let mut lines = 1;
        lines += request.headers.len();

        if let Some(ref body) = request.body {
            if !request.headers.is_empty() {
                lines += 1;
            }
            lines += body.lines().count();
        }

        lines
    }

    pub fn get_used_variables(&self) -> Vec<(String, String)> {
        let Some(request) = self.selected_request() else {
            return Vec::new();
        };

        let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let mut var_names: Vec<String> = Vec::new();

        for cap in re.captures_iter(&request.url) {
            var_names.push(cap[1].to_string());
        }

        for value in request.headers.values() {
            for cap in re.captures_iter(value) {
                var_names.push(cap[1].to_string());
            }
        }

        if let Some(ref body) = request.body {
            for cap in re.captures_iter(body) {
                var_names.push(cap[1].to_string());
            }
        }

        let mut seen = HashSet::new();
        var_names.retain(|name| seen.insert(name.clone()));

        var_names
            .into_iter()
            .filter_map(|name| {
                self.http_file
                    .variables
                    .get(&name)
                    .map(|value| (name, value.clone()))
            })
            .collect()
    }

    pub fn add_history_entry(&mut self, entry: HistoryEntry) {
        self.history.push(entry);
    }

    pub fn toggle_history_view(&mut self) {
        self.history_view_active = !self.history_view_active;
        if self.history_view_active {
            self.focus = Focus::HistoryList;
            if !self.history.is_empty() {
                self.selected_history = self.history.len() - 1;
            }
        } else {
            self.focus = Focus::RequestList;
        }
        self.history_detail_scroll = 0;
    }

    pub fn selected_history_entry(&self) -> Option<&HistoryEntry> {
        self.history.get(self.selected_history)
    }

    pub fn select_previous_history(&mut self) {
        if self.selected_history > 0 {
            self.selected_history -= 1;
            self.history_detail_scroll = 0;
        }
    }

    pub fn select_next_history(&mut self) {
        if self.selected_history < self.history.len().saturating_sub(1) {
            self.selected_history += 1;
            self.history_detail_scroll = 0;
        }
    }

    pub fn scroll_history_detail_up(&mut self) {
        self.history_detail_scroll = self.history_detail_scroll.saturating_sub(1);
    }

    pub fn scroll_history_detail_down(&mut self) {
        self.history_detail_scroll = self.history_detail_scroll.saturating_add(1);
    }
}
