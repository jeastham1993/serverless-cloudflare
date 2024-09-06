CREATE TABLE chats (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE,
    password TEXT
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);