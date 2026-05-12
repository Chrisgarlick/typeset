use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::models::client_profile::ClientProfile;
use crate::models::render_job::{RenderFormat, RenderRequest};
use crate::routes::auth::UserId;
use crate::state::AppState;

/// Render endpoint — renders and returns the file as a download.
pub async fn handle_render(
    State(state): State<AppState>,
    Extension(user): Extension<UserId>,
    Json(req): Json<RenderRequest>,
) -> Result<impl IntoResponse, AppError> {
    let _permit = state
        .render_semaphore
        .acquire()
        .await
        .map_err(|_| AppError::RenderError("Render capacity exceeded".to_string()))?;

    let start = std::time::Instant::now();

    let profile = match &req.client {
        Some(slug) => state
            .db
            .get_client_profile(slug, user.0)
            .await
            .map_err(|_| AppError::NotFound(format!("Client profile '{slug}' not found")))?,
        None => ClientProfile::default_profile(),
    };

    let mut doc = crate::parser::markdown::parse(&req.content);

    if let Some(overrides) = &req.overrides {
        let fm = doc.frontmatter.get_or_insert_with(Default::default);
        if let Some(t) = &overrides.title { fm.title = Some(t.clone()); }
        if let Some(s) = &overrides.subtitle { fm.subtitle = Some(s.clone()); }
        if let Some(r) = &overrides.recipient { fm.recipient = Some(r.clone()); }
        if let Some(d) = &overrides.date { fm.date = Some(d.clone()); }
        if let Some(a) = &overrides.author { fm.author = Some(a.clone()); }
    }

    let branded = crate::brand::engine::BrandedDocument::prepare(doc, profile)
        .map_err(|e| AppError::RenderError(e.to_string()))?;

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

    // Content hash
    let content_hash = {
        let mut hasher = Sha256::new();
        hasher.update(req.content.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    // Log to DB
    let format_str = match req.format {
        RenderFormat::Pdf => "pdf",
        RenderFormat::Docx => "docx",
        RenderFormat::Both => "both",
    };

    if let Err(e) = state
        .db
        .log_render(
            &render_id, user.0, req.client.as_deref(),
            &req.document_type.to_string(), format_str,
            Some(&content_hash), None, None, None, None,
            Some(bytes.len() as i32), render_ms as i32, None,
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
