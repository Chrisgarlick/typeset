CREATE TABLE IF NOT EXISTS client_profiles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL,
    slug        VARCHAR(100) NOT NULL UNIQUE,
    name        VARCHAR(255) NOT NULL,

    -- Branding
    colour_primary      VARCHAR(7) NOT NULL DEFAULT '#000000',
    colour_secondary    VARCHAR(7) NOT NULL DEFAULT '#333333',
    colour_accent       VARCHAR(7) NOT NULL DEFAULT '#0066CC',
    colour_text         VARCHAR(7) NOT NULL DEFAULT '#1A1A1A',
    colour_background   VARCHAR(7) NOT NULL DEFAULT '#FFFFFF',
    colour_table_header VARCHAR(7) NOT NULL DEFAULT '#F5F5F5',
    colour_table_border VARCHAR(7) NOT NULL DEFAULT '#E0E0E0',
    colour_callout_bg   VARCHAR(7) NOT NULL DEFAULT '#F8F9FA',

    -- Typography
    font_heading        VARCHAR(100) NOT NULL DEFAULT 'Helvetica',
    font_body           VARCHAR(100) NOT NULL DEFAULT 'Helvetica',
    font_mono           VARCHAR(100) NOT NULL DEFAULT 'Courier',
    font_size_base      REAL NOT NULL DEFAULT 11.0,
    font_size_h1        REAL NOT NULL DEFAULT 24.0,
    font_size_h2        REAL NOT NULL DEFAULT 18.0,
    font_size_h3        REAL NOT NULL DEFAULT 14.0,
    line_height         REAL NOT NULL DEFAULT 1.5,

    -- Layout
    page_size           VARCHAR(10) NOT NULL DEFAULT 'A4',
    margin_top          REAL NOT NULL DEFAULT 25.4,
    margin_bottom       REAL NOT NULL DEFAULT 25.4,
    margin_left         REAL NOT NULL DEFAULT 25.4,
    margin_right        REAL NOT NULL DEFAULT 25.4,
    paragraph_spacing   REAL NOT NULL DEFAULT 8.0,
    section_spacing     REAL NOT NULL DEFAULT 16.0,

    -- Logo
    logo_light_url      TEXT,
    logo_dark_url       TEXT,
    logo_width          REAL NOT NULL DEFAULT 40.0,
    logo_position       VARCHAR(20) NOT NULL DEFAULT 'left',

    -- Cover page
    cover_enabled       BOOLEAN NOT NULL DEFAULT false,
    cover_template      VARCHAR(20) NOT NULL DEFAULT 'minimal',
    cover_bg_colour     VARCHAR(7) NOT NULL DEFAULT '#000000',
    cover_text_colour   VARCHAR(7) NOT NULL DEFAULT '#FFFFFF',

    -- Header / Footer
    header_enabled      BOOLEAN NOT NULL DEFAULT true,
    header_template     VARCHAR(20) NOT NULL DEFAULT 'logo-left',
    header_border       BOOLEAN NOT NULL DEFAULT true,
    footer_enabled      BOOLEAN NOT NULL DEFAULT true,
    footer_template     VARCHAR(20) NOT NULL DEFAULT 'page-numbers',
    footer_border       BOOLEAN NOT NULL DEFAULT true,

    -- Watermark
    watermark_text      VARCHAR(100),
    watermark_opacity   REAL NOT NULL DEFAULT 0.08,

    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_client_profiles_user_id ON client_profiles(user_id);
CREATE INDEX IF NOT EXISTS idx_client_profiles_slug ON client_profiles(slug);
