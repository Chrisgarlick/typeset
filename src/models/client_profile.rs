use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClientProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub slug: String,
    pub name: String,

    // Branding
    pub colour_primary: String,
    pub colour_secondary: String,
    pub colour_accent: String,
    pub colour_text: String,
    pub colour_background: String,
    pub colour_table_header: String,
    pub colour_table_border: String,
    pub colour_callout_bg: String,

    // Typography
    pub font_heading: String,
    pub font_body: String,
    pub font_mono: String,
    pub font_size_base: f32,
    pub font_size_h1: f32,
    pub font_size_h2: f32,
    pub font_size_h3: f32,
    pub line_height: f32,

    // Layout
    pub page_size: String,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub paragraph_spacing: f32,
    pub section_spacing: f32,

    // Logo
    pub logo_light_url: Option<String>,
    pub logo_dark_url: Option<String>,
    pub logo_width: f32,
    pub logo_position: String,

    // Cover
    pub cover_enabled: bool,
    pub cover_template: String,
    pub cover_bg_colour: String,
    pub cover_text_colour: String,

    // Header / Footer
    pub header_enabled: bool,
    pub header_template: String,
    pub header_border: bool,
    pub footer_enabled: bool,
    pub footer_template: String,
    pub footer_border: bool,

    // Watermark
    pub watermark_text: Option<String>,
    pub watermark_opacity: f32,

    /// Brand name set in the heading font, used where there's no logo image.
    #[serde(default)]
    #[sqlx(default)]
    pub wordmark: Option<String>,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// WCAG contrast ratio between two `#RRGGBB` colours (the theme's own checks).
#[cfg(test)]
pub fn contrast_ratio(a: &str, b: &str) -> Option<f64> {
    fn luminance(hex: &str) -> Option<f64> {
        let hex = hex.strip_prefix('#')?;
        if hex.len() != 6 {
            return None;
        }
        let channel = |i: usize| -> Option<f64> {
            let c = u8::from_str_radix(&hex[i..i + 2], 16).ok()? as f64 / 255.0;
            Some(if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            })
        };
        Some(0.2126 * channel(0)? + 0.7152 * channel(2)? + 0.0722 * channel(4)?)
    }
    let (la, lb) = (luminance(a)?, luminance(b)?);
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    Some((hi + 0.05) / (lo + 0.05))
}

impl ClientProfile {
    /// Default profile used when no client slug is provided
    pub fn default_profile() -> Self {
        let now = chrono::Utc::now();
        ClientProfile {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            slug: "default".to_string(),
            name: "Default".to_string(),
            colour_primary: "#000000".to_string(),
            colour_secondary: "#333333".to_string(),
            colour_accent: "#0066CC".to_string(),
            colour_text: "#1A1A1A".to_string(),
            colour_background: "#FFFFFF".to_string(),
            colour_table_header: "#F5F5F5".to_string(),
            colour_table_border: "#E0E0E0".to_string(),
            colour_callout_bg: "#F8F9FA".to_string(),
            font_heading: "Inter Tight".to_string(),
            font_body: "Inter".to_string(),
            font_mono: "JetBrains Mono".to_string(),
            font_size_base: 10.5,
            font_size_h1: 30.0,
            font_size_h2: 20.0,
            font_size_h3: 14.0,
            line_height: 1.55,
            page_size: "A4".to_string(),
            margin_top: 24.0,
            margin_bottom: 22.0,
            margin_left: 22.0,
            margin_right: 22.0,
            paragraph_spacing: 8.0,
            section_spacing: 16.0,
            logo_light_url: None,
            logo_dark_url: None,
            logo_width: 40.0,
            logo_position: "left".to_string(),
            cover_enabled: false,
            cover_template: "minimal".to_string(),
            cover_bg_colour: "#000000".to_string(),
            cover_text_colour: "#FFFFFF".to_string(),
            header_enabled: true,
            header_template: "logo-left".to_string(),
            header_border: true,
            footer_enabled: true,
            footer_template: "page-numbers".to_string(),
            footer_border: true,
            watermark_text: None,
            watermark_opacity: 0.08,
            wordmark: None,
            created_at: now,
            updated_at: now,
        }
    }
}
