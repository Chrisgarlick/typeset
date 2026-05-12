use axum::{
    extract::{Path, State},
    Json,
};

use crate::error::AppError;
use crate::models::client_profile::{ClientProfile, CreateClientProfile};
use crate::state::AppState;

pub async fn list_clients(
    State(state): State<AppState>,
) -> Result<Json<Vec<ClientProfile>>, AppError> {
    let profiles = state.db.list_client_profiles().await?;
    Ok(Json(profiles))
}

pub async fn get_client(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ClientProfile>, AppError> {
    let profile = state
        .db
        .get_client_profile(&slug)
        .await
        .map_err(|_| AppError::NotFound(format!("Client profile '{slug}' not found")))?;
    Ok(Json(profile))
}

pub async fn create_client(
    State(state): State<AppState>,
    Json(body): Json<CreateClientProfile>,
) -> Result<Json<ClientProfile>, AppError> {
    let profile = state.db.upsert_client_profile(&body).await?;
    Ok(Json(profile))
}

pub async fn delete_client(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted = state.db.delete_client_profile(&slug).await?;
    if deleted {
        Ok(Json(serde_json::json!({ "deleted": true })))
    } else {
        Err(AppError::NotFound(format!(
            "Client profile '{slug}' not found"
        )))
    }
}
