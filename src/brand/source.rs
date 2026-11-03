//! Where a document's brand comes from: the client's published brand in
//! Clients (motion/specs/brand.md), mapped onto the `ClientProfile` the
//! renderers read. Library fonts become per-render font files; logos come
//! from the library. Without a client, Typeset's default look.
//!
//! The mapping is brand.md §8 read backwards; `brand_yaml` writes it forwards
//! for moving old profiles into Clients (`typeset export-brands`).

use serde_json::Value;

use super::assets::{Asset, Logos};
use super::engine::BrandedDocument;
use crate::error::AppError;
use crate::models::client_profile::ClientProfile;
use crate::models::render_job::RenderRequest;
use crate::routes::auth::Caller;
use crate::state::AppState;

/// A brand ready to render with.
pub struct ResolvedBrand {
    pub profile: ClientProfile,
    pub logos: Logos,
    pub fonts: Vec<Asset>,
    pub families: Vec<String>,
    /// The Clients id and brand version, when the brand came from Clients.
    pub source: Option<(String, i64)>,
}

/// Parses the request's document and brands it: the shared start of every
/// render, preview and proof. Returns the document and where its brand came from.
pub async fn branded(state: &AppState, caller: &Caller, req: &RenderRequest) -> Result<(BrandedDocument, Option<(String, i64)>), AppError> {
    let mut doc = crate::parser::parse(&req.input_format, &req.content).map_err(|e| AppError::BadRequest(e.to_string()))?;
    if let Some(overrides) = &req.overrides {
        let fm = doc.frontmatter.get_or_insert_with(Default::default);
        if let Some(t) = &overrides.title {
            fm.title = Some(t.clone());
        }
        if let Some(s) = &overrides.subtitle {
            fm.subtitle = Some(s.clone());
        }
        if let Some(r) = &overrides.recipient {
            fm.recipient = Some(r.clone());
        }
        if let Some(d) = &overrides.date {
            fm.date = Some(d.clone());
        }
        if let Some(a) = &overrides.author {
            fm.author = Some(a.clone());
        }
    }
    let fm = doc.frontmatter.as_ref();
    let slug = req.client.clone().or_else(|| fm.and_then(|f| f.client.clone()));
    let mode = fm.and_then(|f| f.mode.clone());
    let brand = resolve(state, caller, slug.as_deref(), req.brand_version, mode.as_deref()).await?;

    let mut branded = BrandedDocument::prepare(doc, brand.profile).map_err(|e| AppError::RenderError(e.to_string()))?;
    branded.logos = brand.logos;
    branded.fonts = brand.fonts;
    branded.font_families = brand.families;
    branded.images = super::assets::fetch_images(&branded.doc.nodes).await;
    branded.print = req.print;
    Ok((branded, brand.source))
}

/// The brand for a client slug (latest or pinned version, optionally in one
/// of its modes), or the default look when there's no client.
pub async fn resolve(
    state: &AppState,
    caller: &Caller,
    slug: Option<&str>,
    version: Option<i64>,
    mode: Option<&str>,
) -> Result<ResolvedBrand, AppError> {
    let Some(slug) = slug else {
        if version.is_some() {
            return Err(AppError::BadRequest("brand_version needs a client".into()));
        }
        let profile = ClientProfile::default_profile();
        let logos = super::assets::fetch_logos(&profile).await;
        return Ok(ResolvedBrand { profile, logos, fonts: vec![], families: vec![], source: None });
    };
    if !caller.may(slug) {
        return Err(AppError::Forbidden(format!("this key can't use client `{slug}`")));
    }
    let brands = state.brands.as_ref().ok_or_else(|| AppError::Unavailable("Typeset isn't connected to Clients (CLIENTS_URL)".into()))?;
    let info = brands.client(slug).await?;
    let (version, doc) = brands.brand(slug, version).await?;
    let mut profile = profile_from_brand(slug, &doc, mode).map_err(AppError::BadRequest)?;

    // Library files the brand names, by their frozen hash.
    let library = |name: &str| -> Option<(String, String)> {
        let a = &doc["assets"][name];
        Some((a["sha256"].as_str()?.to_string(), a["mime"].as_str().unwrap_or("").to_string()))
    };
    let mut fonts = vec![];
    let mut families = vec![];
    for role in ["heading", "body", "mono"] {
        let t = &doc["type"][role];
        let mut family = None;
        for key in ["font", "italic-font"] {
            let Some(name) = t[key].as_str().and_then(|f| f.strip_prefix("lib:")) else { continue };
            let Some((sha, mime)) = library(name) else { continue };
            let bytes = brands.asset(&sha).await?;
            let Some(found) = family_name(&bytes) else {
                tracing::warn!(client = slug, font = name, "library font without a family name; using the bundled fallback");
                continue;
            };
            let ext = if mime == "font/otf" || bytes.starts_with(b"OTTO") { "otf" } else { "ttf" };
            let file = format!("fonts/{name}.{ext}");
            if !fonts.iter().any(|f: &Asset| f.name == file) {
                fonts.push(Asset { name: file, bytes: bytes.to_vec() });
            }
            if key == "font" {
                family = Some(found.clone());
            }
            if !families.contains(&found) {
                families.push(found);
            }
        }
        if let Some(f) = family {
            match role {
                "heading" => profile.font_heading = f,
                "body" => profile.font_body = f,
                _ => profile.font_mono = f,
            }
        }
    }
    let mut logos = Logos::default();
    for (role, stem) in [("primary", "logo-light"), ("on-dark", "logo-dark")] {
        let Some(name) = doc["logos"][role].as_str().and_then(|l| l.strip_prefix("lib:")) else { continue };
        let Some((sha, mime)) = library(name) else { continue };
        let bytes = brands.asset(&sha).await?;
        let Some(ext) = super::assets::image_extension(&mime, &bytes) else { continue };
        let asset = Asset { name: format!("{stem}.{ext}"), bytes: bytes.to_vec() };
        if role == "primary" {
            logos.light = Some(asset);
        } else {
            logos.dark = Some(asset);
        }
    }
    Ok(ResolvedBrand { profile, logos, fonts, families, source: Some((info.id, version)) })
}

/// A font file's family name (typographic family first).
fn family_name(bytes: &[u8]) -> Option<String> {
    let face = ttf_parser::Face::parse(bytes, 0).ok()?;
    let name = |id| face.names().into_iter().filter(|n| n.name_id == id).find_map(|n| n.to_string());
    name(ttf_parser::name_id::TYPOGRAPHIC_FAMILY).or_else(|| name(ttf_parser::name_id::FAMILY))
}

// --- brand → profile ---------------------------------------------------------------

/// A published brand as a `ClientProfile` (fonts are the names in the brand;
/// `resolve` swaps library fonts for their family names).
pub fn profile_from_brand(slug: &str, doc: &Value, mode: Option<&str>) -> Result<ClientProfile, String> {
    let mut p = ClientProfile::default_profile();
    p.slug = slug.to_string();
    p.name = doc["name"].as_str().unwrap_or(slug).to_string();

    let mut palette = doc["palette"].as_object().cloned().unwrap_or_default();
    // `light` (or nothing) is the base palette, unless the brand has a mode by that name.
    let mode = mode.filter(|m| !m.is_empty()).filter(|m| *m != "light" || doc["modes"].get("light").is_some());
    if let Some(m) = mode {
        let over = doc["modes"].get(m).and_then(Value::as_object).ok_or_else(|| {
            let known: Vec<&str> = doc["modes"].as_object().map(|o| o.keys().map(String::as_str).collect()).unwrap_or_default();
            format!("the brand has no `{m}` mode ({})", if known.is_empty() { "it has none".into() } else { known.join(", ") })
        })?;
        palette.extend(over.clone());
    }
    let paper = palette.get("paper").and_then(Value::as_str).and_then(rgb).unwrap_or([255, 255, 255]);
    let colour = |role: &str, field: &mut String| {
        if let Some(c) = palette.get(role).and_then(Value::as_str).and_then(|h| flat_hex(h, paper)) {
            *field = c;
        }
    };
    colour("paper", &mut p.colour_background);
    colour("ink", &mut p.colour_text);
    colour("ink-muted", &mut p.colour_secondary);
    colour("heading", &mut p.colour_primary);
    colour("accent", &mut p.colour_accent);
    colour("surface", &mut p.colour_table_header);
    colour("rule", &mut p.colour_table_border);
    colour("accent-soft", &mut p.colour_callout_bg);
    colour("inverse", &mut p.cover_bg_colour);
    colour("on-inverse", &mut p.cover_text_colour);

    let font = |role: &str| doc["type"][role]["font"].as_str().map(str::to_string);
    if let Some(f) = font("heading") {
        p.font_heading = f;
    }
    if let Some(f) = font("body") {
        p.font_body = f;
    }
    if let Some(f) = font("mono") {
        p.font_mono = f;
    }
    if let Some(l) = doc["type"]["body"]["leading"].as_f64() {
        p.line_height = l as f32;
    }
    if let Some(w) = doc["wordmark"].as_str() {
        p.wordmark = Some(w.to_string());
    }

    let print = &doc["print"];
    if let Some(page) = print["page"].as_str() {
        p.page_size = page.to_string();
    }
    let m = &print["margins"];
    for (key, field) in [("top", &mut p.margin_top), ("bottom", &mut p.margin_bottom), ("left", &mut p.margin_left), ("right", &mut p.margin_right)] {
        if let Some(v) = length_mm(&m[key]) {
            *field = v;
        }
    }
    if let Some(c) = print["cover"].as_str() {
        p.cover_template = c.to_string();
        p.cover_enabled = true;
    }
    if let Some(h) = print["header"].as_bool() {
        p.header_enabled = h;
    }
    if let Some(f) = print["footer"].as_bool() {
        p.footer_enabled = f;
    }
    p.watermark_text = print["watermark"].as_str().map(str::to_string);
    let size = &print["size"];
    for (key, field) in [("body", &mut p.font_size_base), ("h1", &mut p.font_size_h1), ("h2", &mut p.font_size_h2), ("h3", &mut p.font_size_h3)] {
        if let Some(v) = length_pt(&size[key]) {
            *field = v;
        }
    }
    Ok(p)
}

fn rgb(hex: &str) -> Option<[u8; 3]> {
    let h = hex.strip_prefix('#')?;
    let byte = |i: usize| u8::from_str_radix(h.get(i..i + 2)?, 16).ok();
    match h.len() {
        6 | 8 => Some([byte(0)?, byte(2)?, byte(4)?]),
        _ => None,
    }
}

/// `#RRGGBB`, with any alpha (`#RRGGBBAA`) flattened onto the paper colour:
/// Typst tokens are opaque.
fn flat_hex(hex: &str, paper: [u8; 3]) -> Option<String> {
    let [r, g, b] = rgb(hex)?;
    let h = hex.trim_start_matches('#');
    let a = if h.len() == 8 { u8::from_str_radix(&h[6..8], 16).ok()? as f32 / 255.0 } else { 1.0 };
    let mix = |c: u8, p: u8| (c as f32 * a + p as f32 * (1.0 - a)).round() as u8;
    Some(format!("#{:02X}{:02X}{:02X}", mix(r, paper[0]), mix(g, paper[1]), mix(b, paper[2])))
}

/// `22mm`, `2.2cm`, `1in`, `62pt`, or a number (millimetres).
fn length_mm(v: &Value) -> Option<f32> {
    if let Some(n) = v.as_f64() {
        return Some(n as f32);
    }
    let s = v.as_str()?.trim();
    let (num, factor) = [("mm", 1.0), ("cm", 10.0), ("in", 25.4), ("pt", 25.4 / 72.0)]
        .iter()
        .find_map(|(u, f)| s.strip_suffix(u).map(|n| (n, *f)))
        .unwrap_or((s, 1.0));
    num.trim().parse::<f32>().ok().map(|n| n * factor)
}

/// `10.5pt`, `14px`, or a number (points).
fn length_pt(v: &Value) -> Option<f32> {
    if let Some(n) = v.as_f64() {
        return Some(n as f32);
    }
    let s = v.as_str()?.trim();
    if let Some(px) = s.strip_suffix("px") {
        return px.trim().parse::<f32>().ok().map(|n| n * 0.75);
    }
    s.trim_end_matches("pt").trim().parse().ok()
}

// --- profile → brand (moving old profiles into Clients) ---------------------------

fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn num(v: f32) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// A Typeset profile as brand YAML (motion/specs/brand.md), for pasting into
/// Clients. Logos become `lib:logo-light` / `lib:logo-dark`: upload the files
/// under those names first.
pub fn brand_yaml(p: &ClientProfile) -> String {
    use crate::renderers::typst::{font_family, FontRole};
    let mut y = format!(
        "# From Typeset's profile `{}`. Upload the logo files under the names below, then paste this in.\nbrand: 1\nname: {}\n\npalette:\n",
        p.slug,
        quoted(&p.name)
    );
    for (role, c) in [
        ("paper", &p.colour_background),
        ("ink", &p.colour_text),
        ("ink-muted", &p.colour_secondary),
        ("heading", &p.colour_primary),
        ("accent", &p.colour_accent),
        ("surface", &p.colour_table_header),
        ("rule", &p.colour_table_border),
        ("accent-soft", &p.colour_callout_bg),
        ("inverse", &p.cover_bg_colour),
        ("on-inverse", &p.cover_text_colour),
    ] {
        y.push_str(&format!("  {role}: {}\n", quoted(&c.to_uppercase())));
    }
    y.push_str(&format!(
        "\ntype:\n  heading: {{ font: {} }}\n  body:    {{ font: {}, leading: {} }}\n  mono:    {{ font: {} }}\n",
        quoted(&font_family(&p.font_heading, FontRole::Heading)),
        quoted(&font_family(&p.font_body, FontRole::Body)),
        num(p.line_height),
        quoted(&font_family(&p.font_mono, FontRole::Mono)),
    ));
    if p.logo_light_url.is_some() || p.logo_dark_url.is_some() {
        y.push_str("\nlogos:\n");
        if p.logo_light_url.is_some() {
            y.push_str("  primary: \"lib:logo-light\"\n");
        }
        if p.logo_dark_url.is_some() {
            y.push_str("  on-dark: \"lib:logo-dark\"\n");
        }
    }
    if let Some(w) = p.wordmark.as_deref().filter(|w| !w.is_empty()) {
        y.push_str(&format!("wordmark: {}\n", quoted(w)));
    }
    y.push_str(&format!(
        "\nprint:\n  page: {}\n  margins: {{ top: {}mm, bottom: {}mm, left: {}mm, right: {}mm }}\n  cover: {}\n  header: {}\n  footer: {}\n",
        p.page_size,
        num(p.margin_top),
        num(p.margin_bottom),
        num(p.margin_left),
        num(p.margin_right),
        p.cover_template,
        p.header_enabled,
        p.footer_enabled,
    ));
    if let Some(w) = p.watermark_text.as_deref().filter(|w| !w.is_empty()) {
        y.push_str(&format!("  watermark: {}\n", quoted(w)));
    }
    y.push_str(&format!(
        "  size: {{ body: {}pt, h1: {}pt, h2: {}pt, h3: {}pt }}\n",
        num(p.font_size_base),
        num(p.font_size_h1),
        num(p.font_size_h2),
        num(p.font_size_h3)
    ));
    y
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A brand as Clients publishes it, from brand YAML (here written by hand
    /// in JSON form, as `brand_yaml` would describe it).
    fn published(p: &ClientProfile) -> Value {
        json!({
            "name": p.name,
            "palette": {
                "paper": p.colour_background, "ink": p.colour_text, "ink-muted": p.colour_secondary,
                "heading": p.colour_primary, "accent": p.colour_accent, "surface": p.colour_table_header,
                "rule": p.colour_table_border, "accent-soft": p.colour_callout_bg,
                "inverse": p.cover_bg_colour, "on-inverse": p.cover_text_colour,
            },
            "type": {
                "heading": { "font": p.font_heading }, "body": { "font": p.font_body, "leading": p.line_height }, "mono": { "font": p.font_mono },
            },
            "wordmark": p.wordmark,
            "print": {
                "page": p.page_size,
                "margins": { "top": format!("{}mm", num(p.margin_top)), "bottom": format!("{}mm", num(p.margin_bottom)),
                             "left": format!("{}mm", num(p.margin_left)), "right": format!("{}mm", num(p.margin_right)) },
                "cover": p.cover_template, "header": p.header_enabled, "footer": p.footer_enabled,
                "size": { "body": format!("{}pt", num(p.font_size_base)), "h1": format!("{}pt", num(p.font_size_h1)),
                          "h2": format!("{}pt", num(p.font_size_h2)), "h3": format!("{}pt", num(p.font_size_h3)) },
            },
            "modes": { "dark": { "paper": "#101418", "ink": "#E8E6E1" } },
        })
    }

    #[test]
    fn a_profile_survives_the_trip_through_clients() {
        let original = crate::renderers::typst::tests::gallery_profile();
        let back = profile_from_brand(&original.slug, &published(&original), None).unwrap();
        // Everything the Typst renderer reads comes back the same.
        let typst_fields = |p: &ClientProfile| {
            (
                [&p.colour_background, &p.colour_text, &p.colour_secondary, &p.colour_primary, &p.colour_accent]
                    .map(|c| c.to_uppercase()),
                [&p.colour_table_header, &p.colour_table_border, &p.colour_callout_bg, &p.cover_bg_colour, &p.cover_text_colour]
                    .map(|c| c.to_uppercase()),
                (p.font_heading.clone(), p.font_body.clone(), p.font_mono.clone(), p.wordmark.clone(), p.page_size.clone()),
                [p.font_size_base, p.font_size_h1, p.font_size_h2, p.font_size_h3, p.line_height],
                [p.margin_top, p.margin_bottom, p.margin_left, p.margin_right],
                (p.cover_template.clone(), p.header_enabled, p.footer_enabled),
            )
        };
        assert_eq!(typst_fields(&back), typst_fields(&original));
        // And the YAML for Clients carries the same values.
        let y = brand_yaml(&original);
        assert!(y.contains(&format!("accent: \"{}\"", original.colour_accent.to_uppercase())), "{y}");
        assert!(y.contains("cover: editorial"), "{y}");
    }

    #[test]
    fn modes_alpha_and_units() {
        let doc = json!({
            "name": "Acme",
            "palette": { "paper": "#FFFFFF", "ink": "#000000", "accent": "#336699", "rule": "#00000080" },
            "modes": { "dark": { "paper": "#000000", "ink": "#FFFFFF" } },
            "print": { "margins": { "top": "1in", "left": "2cm" }, "size": { "body": "12px" } },
        });
        let light = profile_from_brand("acme", &doc, None).unwrap();
        assert_eq!(light.colour_table_border, "#7F7F7F", "black at 128/255 over white");
        assert_eq!((light.margin_top, light.margin_left, light.font_size_base), (25.4, 20.0, 9.0));
        let dark = profile_from_brand("acme", &doc, Some("dark")).unwrap();
        assert_eq!((dark.colour_background.as_str(), dark.colour_text.as_str()), ("#000000", "#FFFFFF"));
        assert_eq!(profile_from_brand("acme", &doc, Some("light")).unwrap().colour_background, "#FFFFFF");
        assert!(profile_from_brand("acme", &doc, Some("sepia")).unwrap_err().contains("no `sepia` mode (dark)"));
    }
}
