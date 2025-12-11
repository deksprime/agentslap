//! Application state and logic

use agent_runtime::{Agent, AgentEvent};
use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Agent,
    System,
}

impl Message {
    pub fn user(content: String) -> Self {
        Self {
            role: MessageRole::User,
            content,
            timestamp: Utc::now(),
        }
    }

    pub fn agent(content: String) -> Self {
        Self {
            role: MessageRole::Agent,
            content,
            timestamp: Utc::now(),
        }
    }

    pub fn system(content: String) -> Self {
        Self {
            role: MessageRole::System,
            content,
            timestamp: Utc::now(),
        }
    }
}

pub struct App {
    pub coordinator: Arc<Agent>,
    pub messages: Vec<Message>,
    pub input: String,
    pub scroll: usize,
    pub streaming: bool,
    pub current_response: String,
    stream_rx: Option<mpsc::UnboundedReceiver<StreamUpdate>>,
}

enum StreamUpdate {
    TextChunk(String),
    ToolCall { name: String, params: String },
    ToolResult { name: String, success: bool },
    Done,
    Error(String),
}

impl App {
    pub fn new(coordinator: Arc<Agent>) -> Self {
        Self {
            coordinator,
            messages: vec![
                Message::system("🤖 Agent TUI Ready".to_string()),
                Message::system("Type your message and press Enter. Press Ctrl+Q to quit.".to_string()),
            ],
            input: String::new(),
            scroll: 0,
            streaming: false,
            current_response: String::new(),
            stream_rx: None,
        }
    }

    pub fn input_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn delete_char(&mut self) {
        self.input.pop();
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub async fn submit_message(&mut self) -> Result<()> {
        if self.input.is_empty() || self.streaming {
            return Ok(());
        }

        let user_input = self.input.clone();
        self.input.clear();
        
        // Add user message
        self.messages.push(Message::user(user_input.clone()));

        // Start streaming
        self.streaming = true;
        self.current_response.clear();

        // Create channel for stream updates
        let (tx, rx) = mpsc::unbounded_channel();
        self.stream_rx = Some(rx);

        // Spawn task to handle streaming
        let coordinator = self.coordinator.clone();
        tokio::spawn(async move {
            match coordinator.run_stream(&user_input).await {
                Ok(mut stream) => {
                    while let Some(event_result) = stream.next().await {
                        match event_result {
                            Ok(event) => {
                                let update = match event {
                                    AgentEvent::TextChunk { content } => {
                                        StreamUpdate::TextChunk(content)
                                    }
                                    AgentEvent::ToolCallStart { tool_name, parameters } => {
                                        StreamUpdate::ToolCall {
                                            name: tool_name,
                                            params: parameters.to_string(),
                                        }
                                    }
                                    AgentEvent::ToolCallEnd { tool_name, success, .. } => {
                                        StreamUpdate::ToolResult {
                                            name: tool_name,
                                            success,
                                        }
                                    }
                                    AgentEvent::Done { .. } => {
                                        StreamUpdate::Done
                                    }
                                    AgentEvent::Error { message } => {
                                        StreamUpdate::Error(message)
                                    }
                                    AgentEvent::Thinking { .. } => continue,
                                };
                                
                                if tx.send(update).is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(StreamUpdate::Error(e.to_string()));
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(StreamUpdate::Error(e.to_string()));
                }
            }
        });

        Ok(())
    }

    pub async fn process_stream(&mut self) -> Result<()> {
        if let Some(rx) = &mut self.stream_rx {
            while let Ok(update) = rx.try_recv() {
                match update {
                    StreamUpdate::TextChunk(content) => {
                        self.current_response.push_str(&content);
                    }
                    StreamUpdate::ToolCall { name, params } => {
                        self.current_response.push_str(&format!("\n[🔧 Tool: {} {}]\n", name, params));
                    }
                    StreamUpdate::ToolResult { name, success } => {
                        let status = if success { "✓" } else { "✗" };
                        self.current_response.push_str(&format!("[{} {}]\n", status, name));
                    }
                    StreamUpdate::Done => {
                        self.messages.push(Message::agent(self.current_response.clone()));
                        self.current_response.clear();
                        self.streaming = false;
                        self.stream_rx = None;
                        break;
                    }
                    StreamUpdate::Error(msg) => {
                        self.messages.push(Message::system(format!("❌ Error: {}", msg)));
                        self.current_response.clear();
                        self.streaming = false;
                        self.stream_rx = None;
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn visible_content(&self) -> String {
        if self.streaming && !self.current_response.is_empty() {
            self.current_response.clone()
        } else {
            String::new()
        }
    }
}
