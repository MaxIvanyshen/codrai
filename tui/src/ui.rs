use ratatui::{
    crossterm, layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use std::env;

use crate::app::{App, AppMode};
use crate::color::{ColorType, ColorTheme};

pub fn ui(frame: &mut Frame, app: &mut App) {
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(1), // Title
            Constraint::Min(1), // Main content (placeholder)
            Constraint::Max(1), // Footer
        ])
        .split(frame.area());

    let main_chunk = centered_rect(90, 80, chunks[1]);

    // Title block and widget
    let main_block = Block::default()
        .borders(Borders::NONE)
        .border_type(BorderType::Thick)
        .style(Style::default());

    frame.render_widget(Clear, main_chunk); // Clear the main chunk before rendering
    frame.render_widget(main_block, main_chunk);

    let title = Paragraph::new(Text::styled(
        format!(" * {} ", app.title),
        Style::default()
            .bold()
            .fg(Color::from_u32(*app.color_theme.get_color(ColorType::Accent).unwrap())),
    ));

    frame.render_widget(title, main_chunk);

    // Footer block spanning the entire bottom chunk
    let footer_bg = Color::from_u32(*app.color_theme.get_color(ColorType::Primary).unwrap());
    let footer_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(footer_bg));
    frame.render_widget(footer_block, chunks[2]);

    // Footer content
    let codr_title = Span::styled(
        "CODR",
        Style::default()
            .fg(Color::from_u32(*app.color_theme.get_color(ColorType::Accent).unwrap()))
            .bold(),
    );

    let footer_left_content: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            codr_title,
            Span::raw("  "),
            Span::styled(
                format!(" {} ", env::current_dir().unwrap().display()),
                Style::default().bg(Color::from_u32(*app.color_theme.get_color(ColorType::Danger).unwrap())),
            ),
        ])),
    ];
    let footer_left_list = List::new(footer_left_content)
        .style(Style::default().bg(footer_bg));
    frame.render_widget(footer_left_list, chunks[2]);

    let mode_color = match app.mode {
        AppMode::Normal => Color::from_u32(*app.color_theme.get_color(ColorType::Accent).unwrap()),
        AppMode::Processing => Color::from_u32(*app.color_theme.get_color(ColorType::Success).unwrap()),
    };
    let mode_title = Paragraph::new(Span::styled(
        format!(" {} ", app.mode.to_string()),
        Style::default().bold().bg(mode_color).fg(Color::White),
    ))
    .alignment(ratatui::layout::Alignment::Right);
    frame.render_widget(mode_title, chunks[2]);

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(80), // Main content
            Constraint::Percentage(20), // Input area
        ])
        .split(main_chunk);

    let input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Max(1),
            Constraint::Percentage(95), // Input content area
        ])
        .split(main_layout[1]);

    // Input area
    let input_bg = Color::from_u32(*app.color_theme.get_color(ColorType::Primary).unwrap());
    let input_block = Block::default()
        .borders(Borders::LEFT)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::from_u32(*app.color_theme.get_color(ColorType::Warning).unwrap())))
        .style(Style::default().bg(input_bg));
    frame.render_widget(input_block, main_layout[1]);

    // Split input into lines for multi-line support
    let input_lines: Vec<&str> = app.input.lines().collect();
    let mut display_lines = Vec::new();
    let max_width = input_chunks[1].width as usize - 4; // Subtract 4 for "> " and padding

    if input_lines.is_empty() {
        display_lines.push(Line::from(vec![Span::raw("  > ")]));
    } else {
        for line in input_lines.clone() {
            let mut remaining = line;
            while !remaining.is_empty() {
                let (current_line, rest) = if remaining.len() > max_width {
                    let (part, remainder) = remaining.split_at(max_width);
                    (part, remainder)
                } else {
                    (remaining, "")
                };
                display_lines.push(Line::from(vec![
                    Span::raw("  > ").bold(),
                    Span::styled(
                        current_line.to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]));
                remaining = rest;
            }
        }
    }

    let input_area_list = List::new(display_lines)
        .style(Style::default().bg(input_bg));
    frame.render_widget(input_area_list, input_chunks[1]);

    let mut display_row_index = 0;
    for (i, line) in input_lines.iter().enumerate() {
        if i == input_lines.len() - 1 {
            break; // stop before the last logical line
        }
        display_row_index += (line.len() / max_width) + 1;
    }

    // Add wrapped rows from the last line
    let last_line = input_lines.last().unwrap_or(&"");
    let col = last_line.len() % max_width;
    display_row_index += last_line.len() / max_width;

    let cursor_y = input_chunks[1].top() + display_row_index as u16;
    let cursor_x = input_chunks[1].left() + col as u16 + 4;

    frame.set_cursor_position(Position {
        x: cursor_x,
        y: cursor_y,
    });
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
