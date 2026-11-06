//! `/api/clients`: the clients whose brands Typeset can use. Brands live in
//! Clients now (https://clients.kritano.com); this is a read-only view of it,
//! limited to what the caller's key may use.

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::routes::auth::Caller;
use crate::state::AppState;

const MOVED: &str =
    "Brands are managed in Clients now (https://clients.kritano.com); Typeset only reads them.";

fn brands(state: &AppState) -> Result<&crate::clients::BrandClient, AppError> {
    state.brands.as_deref().ok_or_else(|| {
        AppError::Unavailable("Typeset isn't connected to Clients (CLIENTS_URL)".into())
    })
}

/// Every client this key may use: `[{ slug, name, brand_version }]`.
pub async fn list_clients(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
) -> Result<Json<Vec<Value>>, AppError> {
    let all = brands(&state)?.clients().await?;
    Ok(Json(
        all.into_iter()
            .filter(|c| caller.may(&c.slug))
            .map(|c| json!({ "slug": c.slug, "name": c.name, "brand_version": c.brand_version }))
            .collect(),
    ))
}

/// One client, with its brand as Typeset will use it (`?mode=dark` for a mode).
pub async fn get_client(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(slug): Path<String>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    let brand = crate::brand::source::resolve(
        &state,
        &caller,
        Some(&slug),
        None,
        q.get("mode").map(String::as_str),
    )
    .await?;
    let p = &brand.profile;
    Ok(Json(json!({
        "slug": p.slug,
        "name": p.name,
        "brand_version": brand.source.as_ref().map(|s| s.1),
        "colours": {
            "paper": p.colour_background, "ink": p.colour_text, "ink_muted": p.colour_secondary, "heading": p.colour_primary,
            "accent": p.colour_accent, "surface": p.colour_table_header, "rule": p.colour_table_border, "accent_soft": p.colour_callout_bg,
        },
        "fonts": { "heading": p.font_heading, "body": p.font_body, "mono": p.font_mono },
        "logos": { "light": brand.logos.light.is_some(), "dark": brand.logos.dark.is_some() },
    })))
}

pub async fn create_client() -> AppError {
    AppError::Gone(MOVED.into())
}

pub async fn delete_client() -> AppError {
    AppError::Gone(MOVED.into())
}
