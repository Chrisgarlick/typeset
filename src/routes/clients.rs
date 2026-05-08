use axum::{
    extract::{Path, State},
    Extension, Json,
};

use crate::error::AppError;
use crate::models::client_profile::{ClientProfile, CreateClientProfile};
use crate::routes::auth::UserId;
use crate::state::AppState;

pub async fn list_clients(
    State(state): State<AppState>,
    Extension(user): Extension<UserId>,
) -> Result<Json<Vec<ClientProfile>>, AppError> {
    let profiles = state.db.list_client_profiles(user.0).await?;
    Ok(Json(profiles))
}

pub async fn get_client(
    State(state): State<AppState>,
    Extension(user): Extension<UserId>,
    Path(slug): Path<String>,
) -> Result<Json<ClientProfile>, AppError> {
    let profile = state
        .db
        .get_client_profile(&slug, user.0)
        .await
        .map_err(|_| AppError::NotFound(format!("Client profile '{slug}' not found")))?;
    Ok(Json(profile))
}

pub async fn create_client(
    State(state): State<AppState>,
    Extension(user): Extension<UserId>,
    Json(body): Json<CreateClientProfile>,
) -> Result<Json<ClientProfile>, AppError> {
    let profile = state.db.upsert_client_profile(user.0, &body).await?;
    Ok(Json(profile))
}

pub async fn delete_client(
    State(state): State<AppState>,
    Extension(user): Extension<UserId>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted = state.db.delete_client_profile(&slug, user.0).await?;
    if deleted {
        Ok(Json(serde_json::json!({ "deleted": true })))
    } else {
        Err(AppError::NotFound(format!(
            "Client profile '{slug}' not found"
        )))
    }
}
