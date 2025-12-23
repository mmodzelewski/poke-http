use super::app::{App, Focus};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(frame.area());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[0]);

    render_request_list(frame, app, left_chunks[0]);
    render_variables_panel(frame, app, left_chunks[1]);
    render_response_panel(frame, app, chunks[1]);
}

fn render_request_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .http_file
        .requests
        .iter()
        .map(|req| {
            let method_color = match req.method {
                crate::http::Method::Get => Color::Green,
                crate::http::Method::Post => Color::Yellow,
                crate::http::Method::Put => Color::Blue,
                crate::http::Method::Patch => Color::Cyan,
                crate::http::Method::Delete => Color::Red,
                _ => Color::White,
            };

            let content = Line::from(vec![
                Span::styled(
                    format!("{:7}", req.method),
                    Style::default()
                        .fg(method_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::raw(req.name.as_deref().unwrap_or(&req.url)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let border_style = if app.focus == Focus::RequestList {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Requests ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_variables_panel(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .http_file
        .variables
        .iter()
        .map(|(name, value)| ListItem::new(format!("@{} = {}", name, value)))
        .collect();

    let border_style = if app.focus == Focus::VariablesList {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Variables ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default().with_selected(Some(app.selected_variable));
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_response_panel(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let status_content = if app.loading {
        Line::from(vec![Span::styled(
            "Loading...",
            Style::default().fg(Color::Yellow),
        )])
    } else if let Some(ref response) = app.last_response {
        let status_color = if response.status < 300 {
            Color::Green
        } else if response.status < 400 {
            Color::Yellow
        } else {
            Color::Red
        };

        Line::from(vec![
            Span::styled(
                format!("{} {}", response.status, response.status_text),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:.2?}", response.duration),
                Style::default().fg(Color::DarkGray),
            ),
        ])
    } else {
        Line::from(Span::styled(
            "No response yet. Press Enter to send request.",
            Style::default().fg(Color::DarkGray),
        ))
    };

    let status_block = Paragraph::new(status_content).block(
        Block::default()
            .title(" Status ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(status_block, chunks[0]);

    let body_content = app
        .last_response
        .as_ref()
        .map(|r| format_body(&r.body))
        .unwrap_or_default();

    let border_style = if app.focus == Focus::ResponseBody {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let body_block = Paragraph::new(body_content)
        .block(
            Block::default()
                .title(" Response ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.response_scroll, 0));

    frame.render_widget(body_block, chunks[1]);
}

fn format_body(body: &str) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| body.to_string())
    } else {
        body.to_string()
    }
}
