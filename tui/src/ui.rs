use ratatui::{
    crossterm, layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::env;
use crate::{app::{App, AppMode}, color::ColorType};

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

    let main_chunk = centered_rect(75, 90, chunks[1]);

    // Title block and widget
    let main_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default());

    frame.render_widget(Clear, main_chunk); // Clear the main chunk before rendering
    frame.render_widget(main_block, main_chunk);

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
            Constraint::Percentage(3), // Title area
            Constraint::Percentage(87), // Main content
            Constraint::Percentage(10), // Input area (increased height for multiline)
        ])
        .split(main_chunk);

    let title = Paragraph::new(Text::styled(
        format!(" * {} ", app.title),
        Style::default()
            .bold()
            .fg(Color::from_u32(*app.color_theme.get_color(ColorType::Accent).unwrap())),
    ));

    frame.render_widget(title, main_layout[0]);

    let input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(2), // Input area border
            Constraint::Percentage(98), // Input content area
        ])
        .split(main_layout[2]);

    // Input area
    let input_bg = Color::from_u32(*app.color_theme.get_color(ColorType::Primary).unwrap());
    let input_block = Block::default()
        //.borders(Borders::LEFT)
        //.border_type(BorderType::Thick)
        //.border_style(Style::default().fg(Color::from_u32(*app.color_theme.get_color(ColorType::Warning).unwrap())))
        .style(Style::default().bg(input_bg));

    app.input.set_block(input_block.clone()); // Share the block with the input area
    
    frame.render_widget(
        Block::default()
            .borders(Borders::LEFT).border_type(BorderType::Thick)
            .border_style(Style::default().bg(Color::from_u32(*app.color_theme.get_color(ColorType::Warning).unwrap())).fg(Color::from_u32(*app.color_theme.get_color(ColorType::Warning).unwrap())))
            .style(Style::default().bg(input_bg)),
        input_chunks[0]
    );

    // Render the TextArea in the right chunk
    let input_area = input_chunks[1];
    frame.render_widget(&app.input.clone(), input_area);

    // TODO: make a list of blocks and render them
    for msg in &app.messages {
        let msg_text = Text::from(Span::styled(
            msg.content.clone().unwrap(),
            Style::default().fg(Color::White),
        ));

        let msg_paragraph = Paragraph::new(msg_text)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(msg_paragraph, main_layout[1]);
    }
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
