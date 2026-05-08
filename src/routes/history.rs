use axum::{extract::State, Extension, Json};

use crate::error::AppError;
use crate::models::render_job::{HistoryQuery, RenderHistoryRow};
use crate::routes::auth::UserId;
use crate::state::AppState;

pub async fn get_history(
    State(state): State<AppState>,
    Extension(user): Extension<UserId>,
    axum::extract::Query(query): axum::extract::Query<HistoryQuery>,
) -> Result<Json<Vec<RenderHistoryRow>>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let history = state
        .db
        .get_render_history(
            user.0,
            limit,
            offset,
            query.client.as_deref(),
            query.format.as_deref(),
        )
        .await?;

    Ok(Json(history))
}
