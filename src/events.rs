//! Clients' event feed, polled every 30 seconds with the cursor kept in the
//! database: cached brands and keys are dropped as they change, and a
//! deleted client's render history goes with it.

use crate::state::AppState;

pub async fn poll(state: &AppState) -> anyhow::Result<usize> {
    let Some(brands) = state.brands.as_ref() else {
        return Ok(0);
    };
    let pool = state.db.pool();
    let mut cursor: i64 =
        sqlx::query_scalar("SELECT last_event FROM clients_event_cursor WHERE id = 1")
            .fetch_optional(pool)
            .await?
            .unwrap_or(0);
    let mut handled = 0;
    loop {
        let batch = brands
            .events(cursor)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        if batch.is_empty() {
            return Ok(handled);
        }
        for e in &batch {
            brands.apply(e);
            if e.kind == "client.deleted" {
                let removed = sqlx::query("DELETE FROM render_history WHERE client_id = $1 OR (client_id IS NULL AND client_slug = $2)")
                    .bind(e.client_id.as_deref())
                    .bind(e.slug())
                    .execute(pool)
                    .await?
                    .rows_affected();
                tracing::info!(
                    client = e.slug().unwrap_or(""),
                    removed,
                    "client deleted in Clients: render history removed"
                );
            }
            cursor = e.id;
            sqlx::query("INSERT INTO clients_event_cursor (id, last_event) VALUES (1, $1) ON CONFLICT (id) DO UPDATE SET last_event = EXCLUDED.last_event")
                .bind(cursor)
                .execute(pool)
                .await?;
            handled += 1;
        }
    }
}

/// Polls forever; failures are logged once and retried.
pub async fn run(state: AppState) {
    let mut failing = false;
    loop {
        match poll(&state).await {
            Ok(_) => failing = false,
            Err(e) if !failing => {
                tracing::warn!(error = %e, "can't read Clients' events");
                failing = true;
            }
            Err(_) => {}
        }
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}
