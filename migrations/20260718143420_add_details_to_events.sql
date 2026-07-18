CREATE TYPE event_type AS ENUM ('virtual', 'in_person', 'hybrid');

ALTER TABLE events
    ADD COLUMN details TEXT,
    ADD COLUMN event_type event_type NOT NULL DEFAULT 'in_person',
    ADD COLUMN location TEXT;