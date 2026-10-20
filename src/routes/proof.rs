use axum::{extract::State, Json};

use crate::error::AppError;
use crate::models::client_profile::ClientProfile;
use crate::models::render_job::RenderRequest;
use crate::state::AppState;

/// Thumbnails of every page, for checking a layout before rendering the real
/// file. Takes the same body as `/api/render`; returns
/// `{ "pages": n, "ppi": 40, "images": ["<base64 PNG>", ...] }`.
pub async fn handle_proof(
    State(state): State<AppState>,
    Json(req): Json<RenderRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _permit = state
        .render_semaphore
        .acquire()
        .await
        .map_err(|_| AppError::RenderError("Render capacity exceeded".to_string()))?;

    let profile = match &req.client {
        Some(slug) => state
            .db
            .get_client_profile(slug)
            .await
            .map_err(|_| AppError::NotFound(format!("Client profile '{slug}' not found")))?,
        None => ClientProfile::default_profile(),
    };
    let doc = crate::parser::parse(&req.input_format, &req.content)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let mut branded = crate::brand::engine::BrandedDocument::prepare(doc, profile)
        .map_err(|e| AppError::RenderError(e.to_string()))?;
    branded.logos = crate::brand::assets::fetch_logos(&branded.profile).await;
    branded.images = crate::brand::assets::fetch_images(&branded.doc.nodes).await;

    const PPI: u32 = 40;
    let pages = crate::renderers::typst::render_proof(&branded, PPI)
        .map_err(|e| AppError::RenderError(e.to_string()))?;
    Ok(Json(serde_json::json!({
        "pages": pages.len(),
        "ppi": PPI,
        "images": pages.iter().map(|p| base64(p)).collect::<Vec<_>>(),
    })))
}

/// Standard base64 with padding.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::base64;

    #[test]
    fn encodes_base64() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }
}
