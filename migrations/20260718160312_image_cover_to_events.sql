-- Add migration script here
ALTER TABLE events
    ADD COLUMN cover_image_url TEXT;