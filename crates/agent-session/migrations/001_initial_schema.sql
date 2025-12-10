-- Initial schema for session storage
-- Author: Agent Runtime
-- Date: 2025-12-01

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    -- Unique session identifier
    id TEXT PRIMARY KEY NOT NULL,
    
    -- Full conversation as JSON blob
    conversation_json TEXT NOT NULL,
    
    -- Timestamps (Unix timestamp in seconds)
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    
    -- Metadata for querying
    message_count INTEGER NOT NULL DEFAULT 0,
    estimated_tokens INTEGER NOT NULL DEFAULT 0,
    
    -- Optional metadata JSON
    metadata_json TEXT,
    
    -- Indexable fields for search
    CHECK (length(id) > 0),
    CHECK (length(conversation_json) > 0),
    CHECK (created_at > 0),
    CHECK (updated_at > 0),
    CHECK (message_count >= 0),
    CHECK (estimated_tokens >= 0)
);

-- Index for finding recent sessions
CREATE INDEX IF NOT EXISTS idx_sessions_updated_at 
ON sessions(updated_at DESC);

-- Index for finding old sessions (for cleanup)
CREATE INDEX IF NOT EXISTS idx_sessions_created_at 
ON sessions(created_at DESC);

-- Index for token-based queries
CREATE INDEX IF NOT EXISTS idx_sessions_tokens 
ON sessions(estimated_tokens DESC);

