-- Add migration script here
CREATE TABLE IF NOT EXISTS todos (
    id          TEXT PRIMARY KEY NOT NULL, -- UUID stored as string
    content     TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    finished_at DATETIME,
    finished    BOOLEAN NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_todos_content ON todos (content);
