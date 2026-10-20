//! Images fetched before rendering: the profile's logos and the document's
//! images, written into the compile root for Typst to read.
//!
//! Fetching never fails a render: any failure (a bad URL, a timeout, an
//! unsupported type, too large) is logged and the document renders without
//! that image (logos fall back to the wordmark, images to a placeholder).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::models::client_profile::ClientProfile;
use crate::models::document::DocumentNode;

/// Largest image accepted.
const MAX_BYTES: usize = 5 * 1024 * 1024;
/// Most images fetched for one document.
const MAX_IMAGES: usize = 24;
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

/// Fetch every image the document uses, keyed by URL. Images are named
/// `img-N.ext` in the compile root.
pub async fn fetch_images(nodes: &[DocumentNode]) -> HashMap<String, Asset> {
    let mut urls = Vec::new();
    collect_image_urls(nodes, &mut urls);
    let mut images = HashMap::new();
    for (i, url) in urls.into_iter().take(MAX_IMAGES).enumerate() {
        if let Some(asset) = fetch_asset(Some(&url), &format!("img-{i}")).await {
            images.insert(url, asset);
        }
    }
    images
}

fn collect_image_urls(nodes: &[DocumentNode], urls: &mut Vec<String>) {
    for node in nodes {
        match node {
            DocumentNode::Image { url, .. } if !urls.contains(url) => urls.push(url.clone()),
            DocumentNode::Section { children, .. } => collect_image_urls(children, urls),
            DocumentNode::Columns { children, .. } => {
                for column in children {
                    collect_image_urls(&column.nodes, urls);
                }
            }
            _ => {}
        }
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
            tracing::warn!(url, error = %e, "image not used");
            None
        }
    };
    cache().lock().unwrap().insert(url.to_string(), (Instant::now(), result.clone()));
    result
}

async fn fetch(url: &str) -> anyhow::Result<(Vec<u8>, &'static str)> {
    if !url.starts_with("https://") {
        anyhow::bail!("only https image URLs are fetched");
    }
    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()?;
    let response = client.get(url).send().await?.error_for_status()?;
    if response.content_length().is_some_and(|n| n as usize > MAX_BYTES) {
        anyhow::bail!("image is over {MAX_BYTES} bytes");
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    let bytes = response.bytes().await?;
    if bytes.len() > MAX_BYTES {
        anyhow::bail!("image is over {MAX_BYTES} bytes");
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

    #[test]
    fn collects_each_image_url_once_including_nested_ones() {
        let image = |url: &str| DocumentNode::Image {
            url: url.to_string(),
            alt: String::new(),
            caption: None,
            full_width: false,
        };
        let nodes = vec![
            image("https://a.test/1.png"),
            DocumentNode::Section { style: None, children: vec![image("https://a.test/2.png"), image("https://a.test/1.png")] },
        ];
        let mut urls = Vec::new();
        collect_image_urls(&nodes, &mut urls);
        assert_eq!(urls, vec!["https://a.test/1.png", "https://a.test/2.png"]);
    }

    #[tokio::test]
    async fn refuses_plain_http() {
        assert!(fetch("http://example.com/logo.png").await.is_err());
    }
}
