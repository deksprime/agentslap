//! Application state and logic

use anyhow::Result;
use futures::StreamExt;
use std::pin::Pin;
use tokio::sync::oneshot;
use tokio_stream::Stream;

use crate::client::{AgentClient, StreamEvent};

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

type StreamType = Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>;

pub struct App {
    client: AgentClient,
    agent_id: String,
    pub messages: Vec<Message>,
    pub input: String,
    pub scroll_offset: usize,
    pub streaming_content: String,
    pub is_streaming: bool,
    pub is_connecting: bool,  // New: shows "Connecting..." state
    stream: Option<StreamType>,
    pending_stream: Option<oneshot::Receiver<Result<StreamType>>>,
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
            is_connecting: false,
            stream: None,
            pending_stream: None,
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

    /// Send message - NON-BLOCKING!
    pub fn send_message(&mut self, content: String) {
        // Add user message immediately (instant feedback!)
        self.messages.push(Message {
            role: "user".to_string(),
            content: content.clone(),
            timestamp: chrono::Utc::now(),
        });

        // Mark as connecting
        self.is_connecting = true;
        self.streaming_content.clear();

        // Spawn HTTP call in background task
        let (tx, rx) = oneshot::channel();
        let client = self.client.clone();
        let agent_id = self.agent_id.clone();

        tokio::spawn(async move {
            let result = client.stream_message(&agent_id, &content).await;
            let _ = tx.send(result);
        });

        self.pending_stream = Some(rx);
    }

    pub async fn process_stream_events(&mut self) -> Result<()> {
        // Check if stream arrived from background task
        if let Some(rx) = &mut self.pending_stream {
            match rx.try_recv() {
                Ok(Ok(stream)) => {
                    // Stream ready!
                    self.stream = Some(stream);
                    self.pending_stream = None;
                    self.is_connecting = false;
                    self.is_streaming = true;
                }
                Ok(Err(e)) => {
                    // HTTP error
                    self.streaming_content = format!("Error: {}", e);
                    self.messages.push(Message {
                        role: "agent".to_string(),
                        content: self.streaming_content.clone(),
                        timestamp: chrono::Utc::now(),
                    });
                    self.streaming_content.clear();
                    self.pending_stream = None;
                    self.is_connecting = false;
                }
                Err(oneshot::error::TryRecvError::Empty) => {
                    // Still waiting for HTTP response
                }
                Err(oneshot::error::TryRecvError::Closed) => {
                    // Task died
                    self.pending_stream = None;
                    self.is_connecting = false;
                }
            }
        }

        // Process stream events if we have a stream
        if let Some(stream) = &mut self.stream {
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
