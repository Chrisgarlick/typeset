//! Typst draws a variable font at its default instance only: Outfit's is
//! Thin, so every weight came out hairline. A variable library font is turned
//! into static fonts here, one per hundred on its `wght` axis (100–900), which
//! Typst then picks between by weight. Cached by the file's hash.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use allsorts::binary::read::ReadScope;
use allsorts::font_data::FontData;
use allsorts::tables::variable_fonts::fvar::FvarTable;
use allsorts::tables::{Fixed, FontTableProvider};
use allsorts::tag;

type Instances = Arc<Vec<(u16, Vec<u8>)>>;

fn cache() -> &'static Mutex<HashMap<String, Instances>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Instances>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The static weights of a variable font (`None` for a static font, or one
/// that can't be instanced, which is then used as it is).
pub fn static_weights(sha: &str, bytes: &[u8]) -> Option<Instances> {
    if let Some(hit) = cache().lock().unwrap().get(sha) {
        return Some(hit.clone());
    }
    let found = Arc::new(instances(bytes)?);
    cache()
        .lock()
        .unwrap()
        .insert(sha.to_string(), found.clone());
    Some(found)
}

fn instances(bytes: &[u8]) -> Option<Vec<(u16, Vec<u8>)>> {
    let font = ReadScope::new(bytes).read::<FontData<'_>>().ok()?;
    let provider = font.table_provider(0).ok()?;
    let fvar_data = provider.table_data(tag::FVAR).ok()??;
    let fvar = ReadScope::new(&fvar_data).read::<FvarTable<'_>>().ok()?;
    let axes: Vec<_> = fvar.axes().collect();
    let wght = axes.iter().position(|a| a.axis_tag == tag::WGHT)?;
    let (min, max) = (
        f32::from(axes[wght].min_value),
        f32::from(axes[wght].max_value),
    );
    let mut out = vec![];
    for weight in (100..=900u16)
        .step_by(100)
        .filter(|w| (*w as f32) >= min && (*w as f32) <= max)
    {
        // Every other axis stays at its default.
        let coords: Vec<Fixed> = axes
            .iter()
            .enumerate()
            .map(|(i, a)| {
                if i == wght {
                    Fixed::from(weight as f32)
                } else {
                    a.default_value
                }
            })
            .collect();
        match allsorts::variations::instance(&provider, &coords) {
            Ok((data, _)) => out.push((weight, data)),
            Err(e) => {
                tracing::warn!(weight, error = ?e, "variable font weight not made; using the font as it is");
                return None;
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_variable_font_becomes_one_font_per_weight() {
        let outfit = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/outfit.ttf"
        ))
        .unwrap();
        let weights = instances(&outfit).unwrap();
        assert_eq!(
            weights.iter().map(|w| w.0).collect::<Vec<_>>(),
            [100, 200, 300, 400, 500, 600, 700, 800, 900]
        );
        for (w, data) in weights.iter() {
            let face = ttf_parser::Face::parse(data, 0).unwrap();
            assert_eq!(face.weight().to_number(), *w);
            assert!(face.variation_axes().is_empty(), "static");
        }
    }
}
