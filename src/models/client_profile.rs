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

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request body for creating/updating a client profile
#[derive(Debug, Deserialize)]
pub struct CreateClientProfile {
    pub slug: String,
    pub name: String,

    #[serde(default = "default_colour_primary")]
    pub colour_primary: String,
    #[serde(default = "default_colour_secondary")]
    pub colour_secondary: String,
    #[serde(default = "default_colour_accent")]
    pub colour_accent: String,
    #[serde(default = "default_colour_text")]
    pub colour_text: String,
    #[serde(default = "default_colour_background")]
    pub colour_background: String,
    #[serde(default = "default_colour_table_header")]
    pub colour_table_header: String,
    #[serde(default = "default_colour_table_border")]
    pub colour_table_border: String,
    #[serde(default = "default_colour_callout_bg")]
    pub colour_callout_bg: String,

    #[serde(default = "default_font_heading")]
    pub font_heading: String,
    #[serde(default = "default_font_body")]
    pub font_body: String,
    #[serde(default = "default_font_mono")]
    pub font_mono: String,
    #[serde(default = "default_font_size_base")]
    pub font_size_base: f32,
    #[serde(default = "default_font_size_h1")]
    pub font_size_h1: f32,
    #[serde(default = "default_font_size_h2")]
    pub font_size_h2: f32,
    #[serde(default = "default_font_size_h3")]
    pub font_size_h3: f32,
    #[serde(default = "default_line_height")]
    pub line_height: f32,

    #[serde(default = "default_page_size")]
    pub page_size: String,
    #[serde(default = "default_margin")]
    pub margin_top: f32,
    #[serde(default = "default_margin")]
    pub margin_bottom: f32,
    #[serde(default = "default_margin")]
    pub margin_left: f32,
    #[serde(default = "default_margin")]
    pub margin_right: f32,
    #[serde(default = "default_paragraph_spacing")]
    pub paragraph_spacing: f32,
    #[serde(default = "default_section_spacing")]
    pub section_spacing: f32,

    pub logo_light_url: Option<String>,
    pub logo_dark_url: Option<String>,
    #[serde(default = "default_logo_width")]
    pub logo_width: f32,
    #[serde(default = "default_logo_position")]
    pub logo_position: String,

    #[serde(default)]
    pub cover_enabled: bool,
    #[serde(default = "default_cover_template")]
    pub cover_template: String,
    #[serde(default = "default_cover_bg_colour")]
    pub cover_bg_colour: String,
    #[serde(default = "default_cover_text_colour")]
    pub cover_text_colour: String,

    #[serde(default = "default_true")]
    pub header_enabled: bool,
    #[serde(default = "default_header_template")]
    pub header_template: String,
    #[serde(default = "default_true")]
    pub header_border: bool,
    #[serde(default = "default_true")]
    pub footer_enabled: bool,
    #[serde(default = "default_footer_template")]
    pub footer_template: String,
    #[serde(default = "default_true")]
    pub footer_border: bool,

    pub watermark_text: Option<String>,
    #[serde(default = "default_watermark_opacity")]
    pub watermark_opacity: f32,
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
            font_heading: "Helvetica".to_string(),
            font_body: "Helvetica".to_string(),
            font_mono: "Courier".to_string(),
            font_size_base: 11.0,
            font_size_h1: 24.0,
            font_size_h2: 18.0,
            font_size_h3: 14.0,
            line_height: 1.5,
            page_size: "A4".to_string(),
            margin_top: 25.4,
            margin_bottom: 25.4,
            margin_left: 25.4,
            margin_right: 25.4,
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
            created_at: now,
            updated_at: now,
        }
    }
}

// Default value functions for serde
fn default_colour_primary() -> String { "#000000".to_string() }
fn default_colour_secondary() -> String { "#333333".to_string() }
fn default_colour_accent() -> String { "#0066CC".to_string() }
fn default_colour_text() -> String { "#1A1A1A".to_string() }
fn default_colour_background() -> String { "#FFFFFF".to_string() }
fn default_colour_table_header() -> String { "#F5F5F5".to_string() }
fn default_colour_table_border() -> String { "#E0E0E0".to_string() }
fn default_colour_callout_bg() -> String { "#F8F9FA".to_string() }
fn default_font_heading() -> String { "Helvetica".to_string() }
fn default_font_body() -> String { "Helvetica".to_string() }
fn default_font_mono() -> String { "Courier".to_string() }
fn default_font_size_base() -> f32 { 11.0 }
fn default_font_size_h1() -> f32 { 24.0 }
fn default_font_size_h2() -> f32 { 18.0 }
fn default_font_size_h3() -> f32 { 14.0 }
fn default_line_height() -> f32 { 1.5 }
fn default_page_size() -> String { "A4".to_string() }
fn default_margin() -> f32 { 25.4 }
fn default_paragraph_spacing() -> f32 { 8.0 }
fn default_section_spacing() -> f32 { 16.0 }
fn default_logo_width() -> f32 { 40.0 }
fn default_logo_position() -> String { "left".to_string() }
fn default_cover_template() -> String { "minimal".to_string() }
fn default_cover_bg_colour() -> String { "#000000".to_string() }
fn default_cover_text_colour() -> String { "#FFFFFF".to_string() }
fn default_true() -> bool { true }
fn default_header_template() -> String { "logo-left".to_string() }
fn default_footer_template() -> String { "page-numbers".to_string() }
fn default_watermark_opacity() -> f32 { 0.08 }
