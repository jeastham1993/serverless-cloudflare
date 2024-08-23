-- Migration number: 0001 	 2024-08-23T13:23:23.494Z
CREATE TABLE chats (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE,
    password TEXT
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);