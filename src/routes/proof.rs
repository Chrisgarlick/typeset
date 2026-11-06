use axum::{extract::State, Extension, Json};

use crate::error::AppError;
use crate::routes::auth::Caller;
use crate::models::render_job::RenderRequest;
use crate::state::AppState;

/// Thumbnails of every page, for checking a layout before rendering the real
/// file. Takes the same body as `/api/render`; returns
/// `{ "pages": n, "ppi": 40, "images": ["<base64 PNG>", ...] }`.
pub async fn handle_proof(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Json(req): Json<RenderRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _permit = state
        .render_semaphore
        .acquire()
        .await
        .map_err(|_| AppError::RenderError("Render capacity exceeded".to_string()))?;

    let (branded, _) = crate::brand::source::branded(&state, &caller, &req).await?;

    const PPI: u32 = 40;
    let pages = crate::renderers::typst::render_proof(&branded, PPI)
        .map_err(|e| AppError::RenderError(e.to_string()))?;
    Ok(Json(serde_json::json!({
        "pages": pages.len(),
        "ppi": PPI,
        "images": pages.iter().map(|p| crate::encoding::base64_encode(p)).collect::<Vec<_>>(),
    })))
}
