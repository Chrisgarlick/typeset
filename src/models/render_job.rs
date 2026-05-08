use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RenderRequest {
    /// Client profile slug — uses default profile if None
    pub client: Option<String>,
    pub document_type: DocumentType,
    pub format: RenderFormat,
    /// Raw markdown content
    pub content: String,
    pub overrides: Option<RenderOverrides>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DocumentType {
    Proposal,
    Report,
    Brief,
    Sop,
    Invoice,
    General,
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Proposal => write!(f, "proposal"),
            DocumentType::Report => write!(f, "report"),
            DocumentType::Brief => write!(f, "brief"),
            DocumentType::Sop => write!(f, "sop"),
            DocumentType::Invoice => write!(f, "invoice"),
            DocumentType::General => write!(f, "general"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RenderFormat {
    Pdf,
    Docx,
    Both,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RenderOverrides {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub recipient: Option<String>,
    pub date: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RenderResponse {
    pub success: bool,
    pub render_id: String,
    pub pdf_url: Option<String>,
    pub docx_url: Option<String>,
    pub expires_at: String,
    pub render_ms: u64,
}

/// Query parameters for render history
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub client: Option<String>,
    pub format: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RenderHistoryRow {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub client_slug: Option<String>,
    pub document_type: String,
    pub format: String,
    pub status: String,
    pub content_hash: Option<String>,
    pub frontmatter: Option<serde_json::Value>,
    pub pdf_url: Option<String>,
    pub docx_url: Option<String>,
    pub page_count: Option<i32>,
    pub file_size_bytes: Option<i32>,
    pub render_ms: Option<i32>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
