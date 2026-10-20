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
    branded.print = req.print;

    const PPI: u32 = 40;
    let pages = crate::renderers::typst::render_proof(&branded, PPI)
        .map_err(|e| AppError::RenderError(e.to_string()))?;
    Ok(Json(serde_json::json!({
        "pages": pages.len(),
        "ppi": PPI,
        "images": pages.iter().map(|p| crate::encoding::base64_encode(p)).collect::<Vec<_>>(),
    })))
}
