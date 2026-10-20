-- A text brand mark, set in the heading font, for profiles without a logo image.
ALTER TABLE client_profiles ADD COLUMN IF NOT EXISTS wordmark VARCHAR(100);
