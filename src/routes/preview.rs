use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::error::AppError;
use crate::models::client_profile::ClientProfile;
use crate::models::render_job::RenderRequest;
use crate::state::AppState;

/// Preview endpoint — renders and returns bytes inline (browser-displayable).
pub async fn handle_preview(
    State(state): State<AppState>,
    Json(req): Json<RenderRequest>,
) -> Result<impl IntoResponse, AppError> {
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

    let mut doc = crate::parser::parse(&req.input_format, &req.content)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

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

    let branded = crate::brand::engine::BrandedDocument::prepare(doc, profile)
        .map_err(|e| AppError::RenderError(e.to_string()))?;

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
