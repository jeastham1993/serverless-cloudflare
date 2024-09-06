ALTER TABLE chats ADD COLUMN created_by TEXT NOT NULL DEFAULT 'unknown';

UPDATE chats SET created_by = 'unknown' WHERE created_by = 'unknown';

CREATE INDEX idx_chats_created_by ON chats(created_by);