use std::path::Path;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
};

use crate::{
    app::{AppState, PanelFocus},
    models::{Filter, Mode},
};

pub fn render(frame: &mut Frame<'_>, state: &AppState, data_path: &Path) {
    if state.show_help {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(frame.area());

        render_main(frame, state, chunks[0]);
        render_footer(frame, state, data_path, chunks[1]);
    } else {
        render_main(frame, state, frame.area());
    }

    render_modal(frame, state);
}

fn render_main(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(10)])
        .split(area);

    render_groups_panel(frame, state, chunks[0]);
    render_todo_list(frame, state, chunks[1]);
}

fn panel_block<'a>(title: &'a str, focused: bool) -> Block<'a> {
    let mut block = Block::default().title(title).borders(Borders::ALL);
    if focused {
        block = block.border_style(Style::default().fg(Color::Cyan));
    }
    block
}

fn render_groups_panel(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let focused = state.focused_panel == PanelFocus::Groups;

    if state.groups.is_empty() {
        let empty = Paragraph::new("No groups")
            .block(panel_block("Groups", focused))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(empty, area);
        return;
    }

    let count_width = state
        .groups
        .iter()
        .map(|group| {
            let (completed, total) = state.group_progress(group.id);
            format!("({}/{})", completed, total).len()
        })
        .max()
        .unwrap_or(1);

    let rows: Vec<Row> = state
        .groups
        .iter()
        .map(|group| {
            let (completed, total) = state.group_progress(group.id);
            let progress = format!("({}/{})", completed, total);
            Row::new(vec![
                Cell::from(group.name.clone()),
                Cell::from(format!("{:>width$}", progress, width = count_width)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Min(1), Constraint::Length(count_width as u16)],
    )
    .column_spacing(1)
    .block(panel_block("Groups", focused))
    .row_highlight_style(Style::default().bg(Color::Rgb(0xDD, 0xEE, 0xFF)))
    .highlight_symbol("");

    let mut table_state = TableState::default();
    if focused {
        table_state.select(Some(
            state
                .selected_group_index
                .min(state.groups.len().saturating_sub(1)),
        ));
    }
    frame.render_stateful_widget(table, area, &mut table_state);
}

fn render_todo_list(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let focused = state.focused_panel == PanelFocus::Todos;
    let visible = state.filtered_todo_indices();
    let panel_title = format!("Todos ({})", state.filter.label());

    if visible.is_empty() {
        let text = match state.filter {
            Filter::All => "No todos in this group. Press 'a' to add one.",
            Filter::Open | Filter::Done => "No todos in this filter for selected group.",
        };
        let empty = Paragraph::new(text)
            .block(panel_block(&panel_title, focused))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(empty, area);
        return;
    }

    let rows: Vec<Row> = visible
        .iter()
        .map(|&idx| {
            let todo = &state.todos[idx];
            let prefix = if todo.completed { "[x]" } else { "[ ]" };
            let content = format!("{} {}", prefix, todo.title);
            let date = if todo.completed {
                todo.completed_at
                    .unwrap_or_else(|| todo.created_at.date_naive())
                    .format("%Y-%m-%d")
                    .to_string()
            } else {
                String::new()
            };

            let mut row = Row::new(vec![
                Cell::from(content),
                Cell::from(format!("{:>10}", date)),
            ]);
            if todo.completed {
                row = row.style(Style::default().fg(Color::Green));
            }
            row
        })
        .collect();

    let table = Table::new(rows, [Constraint::Min(1), Constraint::Length(10)])
        .column_spacing(1)
        .block(panel_block(&panel_title, focused))
        .row_highlight_style(Style::default().bg(Color::Rgb(0xDD, 0xEE, 0xFF)))
        .highlight_symbol("");

    let mut table_state = TableState::default();
    if focused {
        table_state.select(Some(
            state
                .selected_todo_index
                .min(visible.len().saturating_sub(1)),
        ));
    }
    frame.render_stateful_widget(table, area, &mut table_state);
}

fn render_footer(frame: &mut Frame<'_>, state: &AppState, data_path: &Path, area: Rect) {
    let mut lines = vec![
        "hjkl: move | a: add | e: edit | x: toggle | d: delete | f: filter | .: help | q: quit"
            .to_string(),
        format!("Data: {}", data_path.display()),
    ];

    if let Some(msg) = &state.status_message {
        lines.push(format!("Status: {}", msg));
    }

    let footer = Paragraph::new(lines.join("\n"))
        .block(Block::default().title("Help").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(footer, area);
}

fn render_modal(frame: &mut Frame<'_>, state: &AppState) {
    match state.mode {
        Mode::Normal => {}
        Mode::AddingTodo | Mode::EditingTodo | Mode::AddingGroup | Mode::EditingGroup => {
            let area = centered_rect(70, 20, frame.area());
            frame.render_widget(Clear, area);

            let title = match state.mode {
                Mode::AddingTodo => "Add Todo",
                Mode::EditingTodo => "Edit Todo",
                Mode::AddingGroup => "Add Group",
                Mode::EditingGroup => "Edit Group",
                _ => "",
            };

            let modal = Paragraph::new(input_with_cursor_line(&state.input_buffer, state.input_cursor))
                .block(
                Block::default()
                    .title(format!("{} (Enter to save, Esc to cancel)", title))
                    .borders(Borders::ALL),
            );
            frame.render_widget(modal, area);
        }
        Mode::ConfirmDeleteTodo | Mode::ConfirmDeleteGroup => {
            let area = centered_rect(60, 20, frame.area());
            frame.render_widget(Clear, area);
            let block = Block::default()
                .title("Confirm Delete")
                .borders(Borders::ALL);
            let inner = block.inner(area);
            frame.render_widget(block, area);

            if inner.height > 0 {
                let bottom = Rect {
                    x: inner.x,
                    y: inner.y + inner.height - 1,
                    width: inner.width,
                    height: 1,
                };
                let parts = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(bottom);

                frame.render_widget(
                    Paragraph::new("confirm(Enter)")
                        .style(Style::default().fg(Color::Red))
                        .alignment(Alignment::Left),
                    parts[0],
                );
                frame.render_widget(
                    Paragraph::new("cancel(Esc)").alignment(Alignment::Right),
                    parts[1],
                );
            }
        }
    }
}

fn input_with_cursor_line(input: &str, cursor: usize) -> Line<'static> {
    let cursor = cursor.min(input.chars().count());
    let mut chars = input.chars();
    let before: String = chars.by_ref().take(cursor).collect();
    let current = chars.next().unwrap_or(' ');
    let after: String = chars.collect();

    Line::from(vec![
        Span::raw(before),
        Span::styled(
            current.to_string(),
            Style::default().bg(Color::Rgb(0xDD, 0xEE, 0xFF)),
        ),
        Span::raw(after),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
