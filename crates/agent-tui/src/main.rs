mod app;
mod client;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal, text::Line, widgets::Paragraph, style::{Color, Style}};
use std::io;

use app::App;
use client::AgentClient;

/// Guard to ensure terminal cleanup on panic or early exit
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

async fn connect_to_server() -> Result<(AgentClient, String)> {
    let client = AgentClient::new("http://localhost:3000");
    let agent = client.create_agent("gpt-4").await?;
    Ok((client, agent.id))
}

async fn show_error_screen(error: &anyhow::Error) -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    
    terminal.draw(|f| {
        let error_text = vec![
            Line::from(""),
            Line::styled("❌ Failed to connect to agent-server", Style::default().fg(Color::Red)),
            Line::from(""),
            Line::from(format!("Error: {}", error)),
            Line::from(""),
            Line::styled("Make sure the server is running:", Style::default().fg(Color::Yellow)),
            Line::from("  cargo run -p agent-server"),
            Line::from(""),
            Line::from("Press any key to exit..."),
        ];
        
        let paragraph = Paragraph::new(error_text);
        f.render_widget(paragraph, f.area());
    })?;
    
    // Wait for keypress
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(_) = event::read()? {
                break;
            }
        }
    }
    
    Ok(())
}

async fn run_app(client: AgentClient, agent_id: String) -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new(client, agent_id);

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        break;
                    }
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            let message = app.input.drain(..).collect();
                            app.send_message(message);  // Non-blocking now!
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Up => {
                        app.scroll_up();
                    }
                    KeyCode::Down => {
                        app.scroll_down();
                    }
                    _ => {}
                }
            }
        }

        // Process any pending stream events
        app.process_stream_events().await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let _guard = TerminalGuard;  // Ensures cleanup on any exit

    // Try to connect to server
    let result = connect_to_server().await;

    match result {
        Ok((client, agent_id)) => {
            run_app(client, agent_id).await?;
        }
        Err(e) => {
            show_error_screen(&e).await?;
        }
    }

    Ok(())
}
