pub mod json;
pub mod markdown;

use crate::models::document::ParsedDocument;
use crate::models::render_job::InputFormat;

pub fn parse(format: &InputFormat, content: &str) -> anyhow::Result<ParsedDocument> {
    match format {
        InputFormat::Markdown => Ok(markdown::parse(content)),
        InputFormat::Json => json::parse(content),
    }
}
