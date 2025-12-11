//! UI rendering with ratatui

use crate::app::{App, Message, MessageRole};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),      // Messages area
            Constraint::Length(3),   // Input area
            Constraint::Length(1),   // Status bar
        ])
        .split(f.area());

    draw_messages(f, app, chunks[0]);
    draw_input(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);
}

fn draw_messages(f: &mut Frame, app: &App, area: Rect) {
    let mut text_lines = Vec::new();

    // Add all messages
    for message in &app.messages {
        let (prefix, style) = match message.role {
            MessageRole::User => (
                "You: ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            MessageRole::Agent => (
                "Agent: ",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            MessageRole::System => (
                "System: ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
            ),
        };

        // Timestamp
        let timestamp = message.timestamp.format("%H:%M:%S").to_string();
        text_lines.push(Line::from(vec![
            Span::styled(format!("[{}] ", timestamp), Style::default().fg(Color::DarkGray)),
            Span::styled(prefix, style),
        ]));

        // Content (wrapped)
        for line in message.content.lines() {
            text_lines.push(Line::from(line.to_string()));
        }
        
        text_lines.push(Line::from(""));  // Empty line between messages
    }

    // Add streaming response if active
    if app.streaming {
        let streaming_content = app.visible_content();
        if !streaming_content.is_empty() {
            text_lines.push(Line::from(vec![
                Span::styled("[", Style::default().fg(Color::DarkGray)),
                Span::styled("●", Style::default().fg(Color::Green).add_modifier(Modifier::RAPID_BLINK)),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::styled("Agent: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]));
            
            for line in streaming_content.lines() {
                text_lines.push(Line::from(line.to_string()));
            }
        }
    }

    let messages_widget = Paragraph::new(text_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("💬 Conversation")
                .style(Style::default().fg(Color::White)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll as u16, 0));

    f.render_widget(messages_widget, area);
}

fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    let input_text = if app.streaming {
        Text::from(Span::styled(
            "⏳ Waiting for response...",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
        ))
    } else {
        Text::from(app.input.as_str())
    };

    let input_widget = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("✏️  Your Message")
                .style(Style::default().fg(Color::White)),
        );

    f.render_widget(input_widget, area);

    // Set cursor position if not streaming
    if !app.streaming {
        f.set_cursor_position((
            area.x + app.input.len() as u16 + 1,
            area.y + 1,
        ));
    }
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let status_text = if app.streaming {
        Span::styled(
            " 🔄 Streaming... | Ctrl+Q to quit ",
            Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        )
    } else {
        Span::styled(
            format!(" ✓ Ready | Messages: {} | Ctrl+Q to quit ", app.messages.len()),
            Style::default().fg(Color::Green).bg(Color::DarkGray),
        )
    };

    let status_widget = Paragraph::new(status_text);
    f.render_widget(status_widget, area);
}
