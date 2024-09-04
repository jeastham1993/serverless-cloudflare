-- Migration: Add created_by column to chats table
ALTER TABLE chats ADD COLUMN created_by TEXT NOT NULL DEFAULT 'unknown';

-- Update existing rows with a default value if needed
UPDATE chats SET created_by = 'unknown' WHERE created_by = 'unknown';

-- Add an index on the created_by column for faster lookups
CREATE INDEX idx_chats_created_by ON chats(created_by);