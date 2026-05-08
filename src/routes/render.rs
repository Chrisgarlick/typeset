use axum::{extract::State, Extension, Json};
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::models::client_profile::ClientProfile;
use crate::models::render_job::{RenderFormat, RenderRequest, RenderResponse};
use crate::routes::auth::UserId;
use crate::state::AppState;

pub async fn handle_render(
    State(state): State<AppState>,
    Extension(user): Extension<UserId>,
    Json(req): Json<RenderRequest>,
) -> Result<Json<RenderResponse>, AppError> {
    // Acquire render semaphore
    let _permit = state
        .render_semaphore
        .acquire()
        .await
        .map_err(|_| AppError::RenderError("Render capacity exceeded".to_string()))?;

    let start = std::time::Instant::now();

    // 1. Fetch client profile (or default)
    let profile = match &req.client {
        Some(slug) => state
            .db
            .get_client_profile(slug, user.0)
            .await
            .map_err(|_| AppError::NotFound(format!("Client profile '{slug}' not found")))?,
        None => ClientProfile::default_profile(),
    };

    // 2. Parse markdown
    let mut doc = crate::parser::markdown::parse(&req.content);

    // 3. Apply overrides
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

    // 4. Brand engine
    let branded = crate::brand::engine::BrandedDocument::prepare(doc, profile)
        .map_err(|e| AppError::RenderError(e.to_string()))?;

    // 5. Render
    let render_id = uuid::Uuid::new_v4().to_string();
    let mut pdf_url = None;
    let mut docx_url = None;
    let mut total_size = 0i32;

    match req.format {
        RenderFormat::Pdf | RenderFormat::Both => {
            let bytes = crate::renderers::pdf::render(&branded)
                .map_err(|e| AppError::RenderError(e.to_string()))?;
            total_size += bytes.len() as i32;
            let key = format!("renders/{}.pdf", render_id);
            let url = state
                .storage
                .upload(&key, bytes, "application/pdf")
                .await
                .map_err(|e| AppError::StorageError(e.to_string()))?;
            pdf_url = Some(url);
        }
        _ => {}
    }

    match req.format {
        RenderFormat::Docx | RenderFormat::Both => {
            let bytes = crate::renderers::docx::render(&branded)
                .map_err(|e| AppError::RenderError(e.to_string()))?;
            total_size += bytes.len() as i32;
            let key = format!("renders/{}.docx", render_id);
            let url = state
                .storage
                .upload(
                    &key,
                    bytes,
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                )
                .await
                .map_err(|e| AppError::StorageError(e.to_string()))?;
            docx_url = Some(url);
        }
        _ => {}
    }

    let render_ms = start.elapsed().as_millis() as u64;
    let expires_at = chrono::Utc::now()
        + chrono::Duration::days(state.config.output_expiry_days);

    // 6. Content hash
    let content_hash = {
        let mut hasher = Sha256::new();
        hasher.update(req.content.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    // 7. Log to DB (non-blocking, errors logged but not returned)
    let format_str = match req.format {
        RenderFormat::Pdf => "pdf",
        RenderFormat::Docx => "docx",
        RenderFormat::Both => "both",
    };

    if let Err(e) = state
        .db
        .log_render(
            &render_id,
            user.0,
            req.client.as_deref(),
            &req.document_type.to_string(),
            format_str,
            Some(&content_hash),
            None,
            pdf_url.as_deref(),
            docx_url.as_deref(),
            None,
            Some(total_size),
            render_ms as i32,
            Some(expires_at),
        )
        .await
    {
        tracing::error!("Failed to log render: {e}");
    }

    Ok(Json(RenderResponse {
        success: true,
        render_id,
        pdf_url,
        docx_url,
        expires_at: expires_at.to_rfc3339(),
        render_ms,
    }))
}
