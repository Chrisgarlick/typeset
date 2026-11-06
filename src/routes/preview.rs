use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Extension, Json,
};

use crate::error::AppError;
use crate::models::render_job::RenderRequest;
use crate::routes::auth::Caller;
use crate::state::AppState;

/// Preview endpoint — renders and returns bytes inline (browser-displayable).
pub async fn handle_preview(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Json(req): Json<RenderRequest>,
) -> Result<impl IntoResponse, AppError> {
    let _permit = state
        .render_semaphore
        .acquire()
        .await
        .map_err(|_| AppError::RenderError("Render capacity exceeded".to_string()))?;

    let (branded, _) = crate::brand::source::branded(&state, &caller, &req).await?;

    let (bytes, content_type, filename) = match req.format {
        crate::models::render_job::RenderFormat::Pdf
        | crate::models::render_job::RenderFormat::Both => {
            let bytes = crate::renderers::typst::render(&branded)
                .map_err(|e| AppError::RenderError(e.to_string()))?;
            (bytes, "application/pdf", "preview.pdf")
        }
        crate::models::render_job::RenderFormat::Docx => {
            let bytes = crate::renderers::docx::render(&branded)
                .map_err(|e| AppError::RenderError(e.to_string()))?;
            (
                bytes,
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "preview.docx",
            )
        }
    };

    let disposition = format!("inline; filename=\"{filename}\"");

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        bytes,
    ))
}
