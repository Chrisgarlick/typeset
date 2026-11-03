-- Brands come from Clients: record which client (by its Clients id, which
-- never changes) and which published brand version styled each render.
ALTER TABLE render_history ADD COLUMN client_id TEXT;
ALTER TABLE render_history ADD COLUMN brand_version INTEGER;
CREATE INDEX idx_render_history_client_id ON render_history (client_id);

-- Where Typeset is in Clients' event feed (client.deleted and the rest).
CREATE TABLE clients_event_cursor (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_event BIGINT NOT NULL
);
