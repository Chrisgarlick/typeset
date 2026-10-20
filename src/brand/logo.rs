//! Logos for the cover and running head, fetched from the profile's URLs.
//!
//! A logo is decoration: any failure (a bad URL, a timeout, an unsupported
//! type) is logged and the document renders without it, falling back to the
//! profile's wordmark.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::models::client_profile::ClientProfile;

/// Largest logo accepted.
const MAX_BYTES: usize = 2 * 1024 * 1024;
/// How long a fetched logo is reused before fetching again.
const CACHE_TTL: Duration = Duration::from_secs(60 * 60);
const TIMEOUT: Duration = Duration::from_secs(8);

/// An image file to write next to the Typst source.
#[derive(Debug, Clone)]
pub struct Asset {
    /// File name inside the compile root, e.g. `logo-light.svg`.
    pub name: String,
    pub bytes: Vec<u8>,
}

/// `light` is for light backgrounds (pages, the editorial cover); `dark` is
/// for dark ones (the dark cover). Either may be missing.
#[derive(Debug, Clone, Default)]
pub struct Logos {
    pub light: Option<Asset>,
    pub dark: Option<Asset>,
}

type Cached = (Instant, Option<(Vec<u8>, &'static str)>);

fn cache() -> &'static Mutex<HashMap<String, Cached>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Cached>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub async fn fetch_logos(profile: &ClientProfile) -> Logos {
    Logos {
        light: fetch_asset(profile.logo_light_url.as_deref(), "logo-light").await,
        dark: fetch_asset(profile.logo_dark_url.as_deref(), "logo-dark").await,
    }
}

async fn fetch_asset(url: Option<&str>, stem: &str) -> Option<Asset> {
    let url = url.map(str::trim).filter(|u| !u.is_empty())?;
    let (bytes, ext) = fetch_cached(url).await?;
    Some(Asset { name: format!("{stem}.{ext}"), bytes })
}

async fn fetch_cached(url: &str) -> Option<(Vec<u8>, &'static str)> {
    if let Some((at, hit)) = cache().lock().unwrap().get(url) {
        if at.elapsed() < CACHE_TTL {
            return hit.clone();
        }
    }
    let result = match fetch(url).await {
        Ok(found) => Some(found),
        Err(e) => {
            tracing::warn!(url, error = %e, "logo not used");
            None
        }
    };
    cache().lock().unwrap().insert(url.to_string(), (Instant::now(), result.clone()));
    result
}

async fn fetch(url: &str) -> anyhow::Result<(Vec<u8>, &'static str)> {
    if !url.starts_with("https://") {
        anyhow::bail!("only https logo URLs are fetched");
    }
    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()?;
    let response = client.get(url).send().await?.error_for_status()?;
    if response.content_length().is_some_and(|n| n as usize > MAX_BYTES) {
        anyhow::bail!("logo is over {MAX_BYTES} bytes");
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    let bytes = response.bytes().await?;
    if bytes.len() > MAX_BYTES {
        anyhow::bail!("logo is over {MAX_BYTES} bytes");
    }
    let ext = image_extension(&content_type, &bytes)
        .ok_or_else(|| anyhow::anyhow!("not a PNG, JPEG, WebP or SVG image ({content_type})"))?;
    Ok((bytes.to_vec(), ext))
}

/// The file extension Typst needs, from the content type or, failing that,
/// the file's first bytes.
fn image_extension(content_type: &str, bytes: &[u8]) -> Option<&'static str> {
    let by_type = match content_type.split(';').next().unwrap_or("").trim() {
        "image/svg+xml" => Some("svg"),
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "image/webp" => Some("webp"),
        _ => None,
    };
    by_type.or_else(|| {
        if bytes.starts_with(b"\x89PNG") {
            Some("png")
        } else if bytes.starts_with(b"\xFF\xD8") {
            Some("jpg")
        } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            Some("webp")
        } else {
            let head = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]).to_ascii_lowercase();
            (head.contains("<svg")).then_some("svg")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_image_types() {
        assert_eq!(image_extension("image/svg+xml; charset=utf-8", b""), Some("svg"));
        assert_eq!(image_extension("application/octet-stream", b"\x89PNG\r\n"), Some("png"));
        assert_eq!(image_extension("text/plain", b"<?xml version=\"1.0\"?><svg>"), Some("svg"));
        assert_eq!(image_extension("text/html", b"<html>"), None);
    }

    #[tokio::test]
    async fn refuses_plain_http() {
        assert!(fetch("http://example.com/logo.png").await.is_err());
    }
}
