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

    // --- Client Profiles ---

    pub async fn get_client_profile(&self, slug: &str) -> Result<ClientProfile, sqlx::Error> {
        sqlx::query_as::<_, ClientProfile>(
            "SELECT * FROM client_profiles WHERE slug = $1",
        )
        .bind(slug)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_client_profiles(&self) -> Result<Vec<ClientProfile>, sqlx::Error> {
        sqlx::query_as::<_, ClientProfile>(
            "SELECT * FROM client_profiles ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn upsert_client_profile(
        &self,
        profile: &crate::models::client_profile::CreateClientProfile,
    ) -> Result<ClientProfile, sqlx::Error> {
        sqlx::query_as::<_, ClientProfile>(
            r#"
            INSERT INTO client_profiles (
                user_id, slug, name,
                colour_primary, colour_secondary, colour_accent, colour_text,
                colour_background, colour_table_header, colour_table_border, colour_callout_bg,
                font_heading, font_body, font_mono,
                font_size_base, font_size_h1, font_size_h2, font_size_h3, line_height,
                page_size, margin_top, margin_bottom, margin_left, margin_right,
                paragraph_spacing, section_spacing,
                logo_light_url, logo_dark_url, logo_width, logo_position,
                cover_enabled, cover_template, cover_bg_colour, cover_text_colour,
                header_enabled, header_template, header_border,
                footer_enabled, footer_template, footer_border,
                watermark_text, watermark_opacity
            ) VALUES (
                $1, $2, $3,
                $4, $5, $6, $7,
                $8, $9, $10, $11,
                $12, $13, $14,
                $15, $16, $17, $18, $19,
                $20, $21, $22, $23, $24,
                $25, $26,
                $27, $28, $29, $30,
                $31, $32, $33, $34,
                $35, $36, $37,
                $38, $39, $40,
                $41, $42
            )
            ON CONFLICT (slug) DO UPDATE SET
                name = EXCLUDED.name,
                colour_primary = EXCLUDED.colour_primary,
                colour_secondary = EXCLUDED.colour_secondary,
                colour_accent = EXCLUDED.colour_accent,
                colour_text = EXCLUDED.colour_text,
                colour_background = EXCLUDED.colour_background,
                colour_table_header = EXCLUDED.colour_table_header,
                colour_table_border = EXCLUDED.colour_table_border,
                colour_callout_bg = EXCLUDED.colour_callout_bg,
                font_heading = EXCLUDED.font_heading,
                font_body = EXCLUDED.font_body,
                font_mono = EXCLUDED.font_mono,
                font_size_base = EXCLUDED.font_size_base,
                font_size_h1 = EXCLUDED.font_size_h1,
                font_size_h2 = EXCLUDED.font_size_h2,
                font_size_h3 = EXCLUDED.font_size_h3,
                line_height = EXCLUDED.line_height,
                page_size = EXCLUDED.page_size,
                margin_top = EXCLUDED.margin_top,
                margin_bottom = EXCLUDED.margin_bottom,
                margin_left = EXCLUDED.margin_left,
                margin_right = EXCLUDED.margin_right,
                paragraph_spacing = EXCLUDED.paragraph_spacing,
                section_spacing = EXCLUDED.section_spacing,
                logo_light_url = EXCLUDED.logo_light_url,
                logo_dark_url = EXCLUDED.logo_dark_url,
                logo_width = EXCLUDED.logo_width,
                logo_position = EXCLUDED.logo_position,
                cover_enabled = EXCLUDED.cover_enabled,
                cover_template = EXCLUDED.cover_template,
                cover_bg_colour = EXCLUDED.cover_bg_colour,
                cover_text_colour = EXCLUDED.cover_text_colour,
                header_enabled = EXCLUDED.header_enabled,
                header_template = EXCLUDED.header_template,
                header_border = EXCLUDED.header_border,
                footer_enabled = EXCLUDED.footer_enabled,
                footer_template = EXCLUDED.footer_template,
                footer_border = EXCLUDED.footer_border,
                watermark_text = EXCLUDED.watermark_text,
                watermark_opacity = EXCLUDED.watermark_opacity,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(TENANT_ID)
        .bind(&profile.slug)
        .bind(&profile.name)
        .bind(&profile.colour_primary)
        .bind(&profile.colour_secondary)
        .bind(&profile.colour_accent)
        .bind(&profile.colour_text)
        .bind(&profile.colour_background)
        .bind(&profile.colour_table_header)
        .bind(&profile.colour_table_border)
        .bind(&profile.colour_callout_bg)
        .bind(&profile.font_heading)
        .bind(&profile.font_body)
        .bind(&profile.font_mono)
        .bind(profile.font_size_base)
        .bind(profile.font_size_h1)
        .bind(profile.font_size_h2)
        .bind(profile.font_size_h3)
        .bind(profile.line_height)
        .bind(&profile.page_size)
        .bind(profile.margin_top)
        .bind(profile.margin_bottom)
        .bind(profile.margin_left)
        .bind(profile.margin_right)
        .bind(profile.paragraph_spacing)
        .bind(profile.section_spacing)
        .bind(&profile.logo_light_url)
        .bind(&profile.logo_dark_url)
        .bind(profile.logo_width)
        .bind(&profile.logo_position)
        .bind(profile.cover_enabled)
        .bind(&profile.cover_template)
        .bind(&profile.cover_bg_colour)
        .bind(&profile.cover_text_colour)
        .bind(profile.header_enabled)
        .bind(&profile.header_template)
        .bind(profile.header_border)
        .bind(profile.footer_enabled)
        .bind(&profile.footer_template)
        .bind(profile.footer_border)
        .bind(&profile.watermark_text)
        .bind(profile.watermark_opacity)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_client_profile(&self, slug: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM client_profiles WHERE slug = $1")
            .bind(slug)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
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
    ) -> Result<(), sqlx::Error> {
        let id: Uuid = render_id.parse().unwrap_or_else(|_| Uuid::new_v4());

        sqlx::query(
            r#"
            INSERT INTO render_history (
                id, user_id, client_slug, document_type, format, status,
                content_hash, frontmatter, pdf_url, docx_url,
                page_count, file_size_bytes, render_ms, expires_at
            ) VALUES (
                $1, $2, $3, $4, $5, 'completed',
                $6, $7, $8, $9,
                $10, $11, $12, $13
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
