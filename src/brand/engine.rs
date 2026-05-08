use crate::models::client_profile::ClientProfile;
use crate::models::document::ParsedDocument;

pub struct BrandedDocument {
    pub profile: ClientProfile,
    pub doc: ParsedDocument,
    pub page: PageDimensions,
    pub colours: ResolvedColours,
}

#[derive(Debug, Clone)]
pub struct PageDimensions {
    pub width_mm: f32,
    pub height_mm: f32,
    pub margin_top_mm: f32,
    pub margin_bottom_mm: f32,
    pub margin_left_mm: f32,
    pub margin_right_mm: f32,
    pub content_width_mm: f32,
    pub content_height_mm: f32,
}

#[derive(Debug, Clone)]
pub struct ResolvedColours {
    pub primary: [f32; 3],
    pub secondary: [f32; 3],
    pub accent: [f32; 3],
    pub text: [f32; 3],
    pub background: [f32; 3],
    pub table_header: [f32; 3],
    pub table_border: [f32; 3],
    pub callout_bg: [f32; 3],
}

impl BrandedDocument {
    pub fn prepare(doc: ParsedDocument, profile: ClientProfile) -> anyhow::Result<Self> {
        let page = resolve_page_dimensions(&profile);
        let colours = resolve_colours(&profile)?;

        Ok(BrandedDocument {
            profile,
            doc,
            page,
            colours,
        })
    }
}

fn resolve_page_dimensions(profile: &ClientProfile) -> PageDimensions {
    let (width, height) = match profile.page_size.as_str() {
        "A4" => (210.0_f32, 297.0_f32),
        "Letter" => (215.9_f32, 279.4_f32),
        "Legal" => (215.9_f32, 355.6_f32),
        _ => (210.0_f32, 297.0_f32),
    };

    let content_width = width - profile.margin_left - profile.margin_right;
    let content_height = height - profile.margin_top - profile.margin_bottom;

    PageDimensions {
        width_mm: width,
        height_mm: height,
        margin_top_mm: profile.margin_top,
        margin_bottom_mm: profile.margin_bottom,
        margin_left_mm: profile.margin_left,
        margin_right_mm: profile.margin_right,
        content_width_mm: content_width,
        content_height_mm: content_height,
    }
}

fn resolve_colours(profile: &ClientProfile) -> anyhow::Result<ResolvedColours> {
    Ok(ResolvedColours {
        primary: hex_to_rgb(&profile.colour_primary)?,
        secondary: hex_to_rgb(&profile.colour_secondary)?,
        accent: hex_to_rgb(&profile.colour_accent)?,
        text: hex_to_rgb(&profile.colour_text)?,
        background: hex_to_rgb(&profile.colour_background)?,
        table_header: hex_to_rgb(&profile.colour_table_header)?,
        table_border: hex_to_rgb(&profile.colour_table_border)?,
        callout_bg: hex_to_rgb(&profile.colour_callout_bg)?,
    })
}

pub fn hex_to_rgb(hex: &str) -> anyhow::Result<[f32; 3]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        anyhow::bail!("Invalid hex colour: #{hex}");
    }
    let r = u8::from_str_radix(&hex[0..2], 16)? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16)? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16)? as f32 / 255.0;
    Ok([r, g, b])
}

/// Convert mm to PDF points (1mm = 2.834645669 pt)
pub fn mm_to_pt(mm: f32) -> f32 {
    mm * 2.834_645_7
}

/// Approximate character-count text wrapping
pub fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > max_chars && !current.is_empty() {
            lines.push(current.trim().to_string());
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current.trim().to_string());
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb() {
        let rgb = hex_to_rgb("#FF0000").unwrap();
        assert!((rgb[0] - 1.0).abs() < 0.01);
        assert!((rgb[1] - 0.0).abs() < 0.01);
        assert!((rgb[2] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_hex_to_rgb_no_hash() {
        let rgb = hex_to_rgb("00FF00").unwrap();
        assert!((rgb[1] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hex_to_rgb_invalid() {
        assert!(hex_to_rgb("invalid").is_err());
    }

    #[test]
    fn test_wrap_text() {
        let text = "This is a test of the word wrapping function";
        let lines = wrap_text(text, 20);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(line.len() <= 25); // allow slight overflow for last word
        }
    }

    #[test]
    fn test_wrap_empty() {
        let lines = wrap_text("", 80);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "");
    }

    #[test]
    fn test_mm_to_pt() {
        let pt = mm_to_pt(25.4); // 1 inch = 72pt
        assert!((pt - 72.0).abs() < 0.1);
    }
}
