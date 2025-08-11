use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Styled, Stylize},
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
            Constraint::Percentage(10), // Input area
        ])
        .split(main_chunk);

    let title = Paragraph::new(Text::styled(
        format!(" * {} ", app.title),
        Style::default()
            .bold()
            .fg(Color::from_u32(*app.color_theme.get_color(ColorType::Accent).unwrap())),
    ));
    frame.render_widget(title, main_layout[0]);

    create_input_area(app, frame, &main_layout[2]);

    let mut y_offset = 1;
    let msg_padding = 1; // Padding between messages

    for (i, msg) in app.messages.iter().enumerate() {
        let mut border_color = Color::from_u32(*app.color_theme.get_color(ColorType::Warning).unwrap());
        if *msg.role.as_ref().unwrap() == openai::Role::Assistant {
            border_color = Color::from_u32(*app.color_theme.get_color(ColorType::Danger).unwrap());
        }
        let content = msg.content.clone().unwrap_or_else(|| "".to_string());
        let lines: Vec<&str> = content.split('\n').collect();
        let line_count = lines.len() as u16; // Number of lines
        let min_height = 2; // Minimum height for visibility
        let max_lines_per_message = 5; // Cap the number of lines to display
        let height = (line_count.min(max_lines_per_message)).max(min_height); // Dynamic height with caps

        let mut msg_bg_style = Style::default().bg(Color::from_u32(*app.color_theme.get_color(ColorType::Primary).unwrap()));
        if *msg.role.as_ref().unwrap() == openai::Role::User {
            msg_bg_style = Style::default(); // No background for user messages
        }

        let message_block = Block::default()
            .style(msg_bg_style);

        let text = Paragraph::new(Text::from(content))
            .wrap(Wrap { trim: true })
            .block(message_block);
        let area = Rect {
            x: main_layout[1].x,
            y: main_layout[1].y + y_offset,
            width: main_layout[1].width,
            height, // Use calculated height based on line count
        };

        let msg_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(2),
                Constraint::Percentage(98),
            ])
            .split(area);

        let border_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(msg_chunks[0]);

        frame.render_widget(
            Block::default()
                .style(Style::default().bg(border_color)),
            border_chunk[0],
        );
        frame.render_widget(
            Block::default()
            .style(msg_bg_style),
            border_chunk[1],
        );
        frame.render_widget(text, msg_chunks[1]);

        y_offset += height + msg_padding; // Update offset for the next message
    }
}

fn create_input_area(app: &mut App, frame: &mut Frame, area: &Rect) {
    let input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(2), // Input area border
            Constraint::Percentage(98), // Input content area
        ])
        .split(*area);

    let input_bg = Color::from_u32(*app.color_theme.get_color(ColorType::Primary).unwrap());

    let input_block = Block::default()
        .style(Style::default().bg(input_bg));

    app.input.set_block(input_block.clone()); 
    
    frame.render_widget(
        Block::default()
            .borders(Borders::LEFT).border_type(BorderType::Thick)
            .border_style(Style::default().bg(Color::from_u32(*app.color_theme.get_color(ColorType::Warning).unwrap())).fg(Color::from_u32(*app.color_theme.get_color(ColorType::Warning).unwrap())))
            .style(Style::default().bg(input_bg)),
        input_chunks[0]
    );

    let input_area = input_chunks[1];
    frame.render_widget(&app.input.clone(), input_area);
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

