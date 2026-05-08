CREATE TABLE IF NOT EXISTS render_history (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,
    client_slug     VARCHAR(100),
    document_type   VARCHAR(50) NOT NULL DEFAULT 'general',
    format          VARCHAR(10) NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',

    -- Input
    content_hash    VARCHAR(64),
    frontmatter     JSONB,

    -- Output
    pdf_url         TEXT,
    docx_url        TEXT,
    page_count      INTEGER,
    file_size_bytes INTEGER,

    -- Timing
    render_ms       INTEGER,
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_render_history_user_id ON render_history(user_id);
CREATE INDEX IF NOT EXISTS idx_render_history_created_at ON render_history(created_at DESC);
