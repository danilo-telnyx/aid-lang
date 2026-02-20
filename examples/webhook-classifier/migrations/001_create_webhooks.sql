-- Webhook Classifier Schema
-- AID Language Demo Application

CREATE TABLE IF NOT EXISTS webhooks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL DEFAULT 'unknown',
    event_type TEXT NOT NULL DEFAULT 'custom',
    payload TEXT NOT NULL,
    classification TEXT NOT NULL DEFAULT 'unclassified',
    priority TEXT NOT NULL DEFAULT 'medium',
    confidence REAL NOT NULL DEFAULT 0.0,
    processed BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS classification_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    webhook_id INTEGER NOT NULL REFERENCES webhooks(id),
    old_classification TEXT,
    new_classification TEXT NOT NULL,
    changed_by TEXT NOT NULL DEFAULT 'system',
    reason TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at DATETIME
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_webhooks_classification ON webhooks(classification);
CREATE INDEX IF NOT EXISTS idx_webhooks_priority ON webhooks(priority);
CREATE INDEX IF NOT EXISTS idx_webhooks_created_at ON webhooks(created_at);
CREATE INDEX IF NOT EXISTS idx_webhooks_source ON webhooks(source);
CREATE INDEX IF NOT EXISTS idx_classification_log_webhook ON classification_log(webhook_id);
