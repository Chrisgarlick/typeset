use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::routes::auth::Caller;
use crate::models::render_job::{RenderFormat, RenderRequest};
use crate::state::AppState;

/// Render endpoint — renders and returns the file as a download.
pub async fn handle_render(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Json(req): Json<RenderRequest>,
) -> Result<impl IntoResponse, AppError> {
    let _permit = state
        .render_semaphore
        .acquire()
        .await
        .map_err(|_| AppError::RenderError("Render capacity exceeded".to_string()))?;

    let start = std::time::Instant::now();

    let (branded, source) = crate::brand::source::branded(&state, &caller, &req).await?;
    let client_slug = branded.profile.slug.clone();

    let (bytes, content_type, ext) = match req.format {
        RenderFormat::Pdf | RenderFormat::Both => {
            let b = crate::renderers::typst::render(&branded)
                .map_err(|e| AppError::RenderError(e.to_string()))?;
            (b, "application/pdf", "pdf")
        }
        RenderFormat::Docx => {
            let b = crate::renderers::docx::render(&branded)
                .map_err(|e| AppError::RenderError(e.to_string()))?;
            (b, "application/vnd.openxmlformats-officedocument.wordprocessingml.document", "docx")
        }
    };

    let render_ms = start.elapsed().as_millis() as u64;
    let render_id = uuid::Uuid::new_v4().to_string();

    let content_hash = {
        let mut hasher = Sha256::new();
        hasher.update(req.content.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let format_str = match req.format {
        RenderFormat::Pdf => "pdf",
        RenderFormat::Docx => "docx",
        RenderFormat::Both => "both",
    };

    if let Err(e) = state
        .db
        .log_render(
            &render_id, source.as_ref().map(|_| client_slug.as_str()),
            &req.document_type.to_string(), format_str,
            Some(&content_hash), None, None, None, None,
            Some(bytes.len() as i32), render_ms as i32, None,
            source.as_ref().map(|(id, v)| (id.as_str(), *v)),
        )
        .await
    {
        tracing::error!("Failed to log render: {e}");
    }

    let disposition = format!("attachment; filename=\"document.{ext}\"");

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        bytes,
    ))
}
