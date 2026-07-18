-- Add migration script here
ALTER TABLE events
    ADD COLUMN meeting_url TEXT;