//! Application state and logic

use anyhow::Result;
use futures::StreamExt;
use std::pin::Pin;
use tokio_stream::Stream;

use crate::client::{AgentClient, StreamEvent};

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct App {
    client: AgentClient,
    agent_id: String,
    pub messages: Vec<Message>,
    pub input: String,
    pub scroll_offset: usize,
    pub streaming_content: String,
    pub is_streaming: bool,
    stream: Option<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>>,
}

impl App {
    pub fn new(client: AgentClient, agent_id: String) -> Self {
        Self {
            client,
            agent_id,
            messages: Vec::new(),
            input: String::new(),
            scroll_offset: 0,
            streaming_content: String::new(),
            is_streaming: false,
            stream: None,
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub async fn send_message(&mut self, content: String) -> Result<()> {
        // Add user message
        self.messages.push(Message {
            role: "user".to_string(),
            content: content.clone(),
            timestamp: chrono::Utc::now(),
        });

        // Start streaming response
        self.streaming_content.clear();
        self.is_streaming = true;

        let stream = self.client.stream_message(&self.agent_id, &content).await?;
        self.stream = Some(stream);

        Ok(())
    }

    pub async fn process_stream_events(&mut self) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            // Try to get one event (non-blocking)
            if let Some(event_result) = stream.next().await {
                match event_result {
                    Ok(event) => {
                        match event {
                            StreamEvent::TextChunk { content } => {
                                self.streaming_content.push_str(&content);
                            }
                            StreamEvent::ToolCall { name, .. } => {
                                self.streaming_content.push_str(&format!("\n[🔧 Tool: {}]\n", name));
                            }
                            StreamEvent::ToolResult { name, success } => {
                                let status = if success { "✓" } else { "✗" };
                                self.streaming_content.push_str(&format!("[{} {}]\n", status, name));
                            }
                            StreamEvent::Done { .. } => {
                                // Add agent message
                                self.messages.push(Message {
                                    role: "agent".to_string(),
                                    content: self.streaming_content.clone(),
                                    timestamp: chrono::Utc::now(),
                                });
                                self.streaming_content.clear();
                                self.is_streaming = false;
                                self.stream = None;
                            }
                            StreamEvent::Error { message } => {
                                self.streaming_content.push_str(&format!("\n[Error: {}]\n", message));
                                self.is_streaming = false;
                                self.stream = None;
                            }
                        }
                    }
                    Err(e) => {
                        self.streaming_content.push_str(&format!("\n[Stream error: {}]\n", e));
                        self.is_streaming = false;
                        self.stream = None;
                    }
                }
            }
        }

        Ok(())
    }
}
