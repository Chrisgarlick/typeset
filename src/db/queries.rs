use sqlx::PgPool;
use uuid::Uuid;

use crate::models::client_profile::ClientProfile;
use crate::models::render_job::RenderHistoryRow;

/// Single-tenant model: all rows are owned by a fixed nil UUID, no per-user
/// scoping. The user_id column is retained in the schema so a future
/// multi-user migration is straightforward (backfill real UUIDs).
const TENANT_ID: Uuid = Uuid::nil();

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Database { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // --- Client profiles (read only: brands live in Clients; `typeset export-brands`) ---

    pub async fn list_client_profiles(&self) -> Result<Vec<ClientProfile>, sqlx::Error> {
        sqlx::query_as::<_, ClientProfile>("SELECT * FROM client_profiles ORDER BY name")
            .fetch_all(&self.pool)
            .await
    }

    // --- Render History ---

    pub async fn log_render(
        &self,
        render_id: &str,
        client_slug: Option<&str>,
        document_type: &str,
        format: &str,
        content_hash: Option<&str>,
        frontmatter: Option<&serde_json::Value>,
        pdf_url: Option<&str>,
        docx_url: Option<&str>,
        page_count: Option<i32>,
        file_size_bytes: Option<i32>,
        render_ms: i32,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        brand: Option<(&str, i64)>,
    ) -> Result<(), sqlx::Error> {
        let id: Uuid = render_id.parse().unwrap_or_else(|_| Uuid::new_v4());

        sqlx::query(
            r#"
            INSERT INTO render_history (
                id, user_id, client_slug, document_type, format, status,
                content_hash, frontmatter, pdf_url, docx_url,
                page_count, file_size_bytes, render_ms, expires_at,
                client_id, brand_version
            ) VALUES (
                $1, $2, $3, $4, $5, 'completed',
                $6, $7, $8, $9,
                $10, $11, $12, $13,
                $14, $15
            )
            "#,
        )
        .bind(id)
        .bind(TENANT_ID)
        .bind(client_slug)
        .bind(document_type)
        .bind(format)
        .bind(content_hash)
        .bind(frontmatter)
        .bind(pdf_url)
        .bind(docx_url)
        .bind(page_count)
        .bind(file_size_bytes)
        .bind(render_ms)
        .bind(expires_at)
        .bind(brand.map(|b| b.0))
        .bind(brand.map(|b| b.1 as i32))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_render_history(
        &self,
        limit: i64,
        offset: i64,
        client_filter: Option<&str>,
        format_filter: Option<&str>,
    ) -> Result<Vec<RenderHistoryRow>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM render_history WHERE 1=1");
        let mut param_idx = 1;

        if client_filter.is_some() {
            query.push_str(&format!(" AND client_slug = ${param_idx}"));
            param_idx += 1;
        }
        if format_filter.is_some() {
            query.push_str(&format!(" AND format = ${param_idx}"));
            param_idx += 1;
        }

        query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1
        ));

        let mut q = sqlx::query_as::<_, RenderHistoryRow>(&query);

        if let Some(client) = client_filter {
            q = q.bind(client);
        }
        if let Some(format) = format_filter {
            q = q.bind(format);
        }

        q = q.bind(limit).bind(offset);

        q.fetch_all(&self.pool).await
    }
}
