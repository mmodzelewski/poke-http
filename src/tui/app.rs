use crate::client::Response;
use crate::http::{HttpFile, Request};
use regex::Regex;
use std::collections::HashSet;

pub struct App {
    pub http_file: HttpFile,
    pub selected: usize,
    pub selected_variable: usize,
    pub last_response: Option<Response>,
    pub should_quit: bool,
    pub focus: Focus,
    pub response_scroll: u16,
    pub request_details_scroll: u16,
    pub request_details_visible_height: u16,
    pub loading: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    RequestList,
    ResponseBody,
    RequestDetails,
    VariablesList,
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
            request_details_scroll: 0,
            request_details_visible_height: 0,
            loading: false,
        }
    }

    pub fn selected_request(&self) -> Option<&Request> {
        self.http_file.requests.get(self.selected)
    }

    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.selected_variable = 0;
            self.request_details_scroll = 0;
        }
    }

    pub fn select_next(&mut self) {
        if self.selected < self.http_file.requests.len().saturating_sub(1) {
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
}
