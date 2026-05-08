use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Check database connectivity
    let db_ok = sqlx::query("SELECT 1")
        .execute(state.db.pool())
        .await
        .is_ok();

    if db_ok {
        Ok(Json(json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
            "database": "connected"
        })))
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Database connection failed".to_string(),
        ))
    }
}
