//! Communication tools for agent-to-agent coordination

pub mod delegate_task;
pub mod escalate;
pub mod broadcast_to_team;
pub mod send_message;

pub use delegate_task::DelegateTaskTool;
pub use escalate::EscalateTool;
pub use broadcast_to_team::BroadcastToTeamTool;
pub use send_message::SendMessageTool;
