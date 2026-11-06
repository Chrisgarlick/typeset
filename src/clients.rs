//! brand-client: how Typeset and Motion talk to Clients (SPEC.md §4–5).
//!
//! Shared with `clients/crates/brand-client` (a copy: the clients repo is
//! private and Typeset's image builds without it). Keep the two in step;
//! this copy is written for edition 2021 (no `if let` chains).
//!
//! - `brand(slug, None)` → the latest published brand, revalidated with an
//!   ETag; pinned versions are cached for good (memory and disk). If Clients
//!   can't be reached, the last brand seen is used (decision D21).
//! - `asset(sha256)` → library files by hash, verified, in a size-capped disk cache.
//! - `authenticate(header)` → the caller behind `Authorization: Bearer ck_…`,
//!   checked with Clients and cached for 60 seconds.
//! - `events(after)` → the change feed; renderers poll it every 30 seconds.
//! - `handoff(code, app)` → swaps a "Sign in with Clients" code.

// A shared copy: not every method is used here.
#![allow(dead_code)]

use serde::Deserialize;
use serde_json::{Value, json};
use sha2::Digest;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct Config {
    /// e.g. `http://127.0.0.1:8130` (same box) or `https://clients.kritano.com`.
    pub base: String,
    /// A `service` key from Clients.
    pub service_key: String,
    /// Holds `brands/` and `assets/`.
    pub cache_dir: PathBuf,
    /// The asset cache's size limit (oldest files go first).
    pub asset_cache_bytes: u64,
}

/// The app calling a renderer.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Caller {
    /// The key's id (not secret).
    pub id: String,
    /// The key's name, e.g. `kritano`.
    pub key: String,
    pub kind: String,
    /// The client slugs this key may use; `None` means all.
    pub clients: Option<Vec<String>>,
}

impl Caller {
    pub fn may(&self, slug: &str) -> bool {
        self.clients.as_ref().is_none_or(|c| c.iter().any(|s| s == slug))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    pub id: i64,
    #[serde(rename = "type")]
    pub kind: String,
    pub client_id: Option<String>,
    pub payload: Value,
    pub created_at: i64,
}

impl Event {
    /// The client slug the event is about, if any.
    pub fn slug(&self) -> Option<&str> {
        self.payload["slug"].as_str()
    }
}

/// A client as Clients describes it. `id` never changes; slugs can be reused
/// after a client is deleted, so renderers key their data by `id`.
#[derive(Debug, Clone, Deserialize)]
pub struct ClientInfo {
    pub id: String,
    pub slug: String,
    pub name: String,
    /// The latest published brand version, if any.
    pub brand_version: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryAsset {
    pub name: String,
    pub kind: String,
    pub sha256: String,
    pub mime: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub meta: Value,
}

#[derive(Debug)]
pub enum Error {
    /// No key, or Clients says it's unknown or revoked.
    Unauthorized,
    /// The key can't use this client.
    Forbidden(String),
    NotFound(String),
    /// Clients can't be reached and nothing usable is cached.
    Unavailable(String),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unauthorized => f.write_str("unknown or revoked key"),
            Error::Forbidden(m) | Error::NotFound(m) | Error::Unavailable(m) | Error::Other(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for Error {}

const KEY_TTL: Duration = Duration::from_secs(60);

pub struct BrandClient {
    cfg: Config,
    http: reqwest::Client,
    /// slug → (latest version, document).
    latest: Mutex<HashMap<String, (i64, Arc<Value>)>>,
    /// (slug, version) → document.
    versions: Mutex<HashMap<(String, i64), Arc<Value>>>,
    /// SHA-256 of a key → (checked at, caller).
    keys: Mutex<HashMap<String, (Instant, Option<Caller>)>>,
    /// slug → client (dropped on `client.deleted`).
    clients: Mutex<HashMap<String, ClientInfo>>,
}

fn sha256(data: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(data))
}

impl BrandClient {
    pub fn new(cfg: Config) -> BrandClient {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(3))
            .build()
            .expect("http client");
        BrandClient {
            cfg,
            http,
            latest: Default::default(),
            versions: Default::default(),
            keys: Default::default(),
            clients: Default::default(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.cfg.base.trim_end_matches('/'))
    }

    fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.http.get(self.url(path)).bearer_auth(&self.cfg.service_key)
    }

    async fn check(r: reqwest::Response) -> Result<reqwest::Response, Error> {
        match r.status().as_u16() {
            200..=299 | 304 => Ok(r),
            401 => Err(Error::Other("Clients refused this service key".into())),
            403 => Err(Error::Other("this key isn't a service key".into())),
            404 => Err(Error::NotFound(
                r.json::<Value>().await.ok().and_then(|v| v["detail"].as_str().map(str::to_string)).unwrap_or_else(|| "not found".into()),
            )),
            s => Err(Error::Unavailable(format!("Clients answered {s}"))),
        }
    }

    // --- keys ------------------------------------------------------------------

    /// Who a `ck_…` key belongs to (`None` if unknown or revoked). Cached 60s.
    pub async fn introspect(&self, token: &str) -> Result<Option<Caller>, Error> {
        let h = sha256(token.as_bytes());
        if let Some((at, c)) = self.keys.lock().unwrap().get(&h) {
            if at.elapsed() < KEY_TTL {
                return Ok(c.clone());
            }
        }
        let r = self
            .http
            .post(self.url("/v1/auth/introspect"))
            .bearer_auth(&self.cfg.service_key)
            .json(&json!({ "token": token }))
            .send()
            .await;
        let r = Self::check(r.map_err(|e| Error::Unavailable(format!("Clients can't be reached: {e}")))?).await?;
        let v: Value = r.json().await.map_err(|e| Error::Other(e.to_string()))?;
        let caller = if v["active"] == true { serde_json::from_value(v).ok() } else { None };
        self.keys.lock().unwrap().insert(h, (Instant::now(), caller.clone()));
        Ok(caller)
    }

    /// The caller behind an `Authorization` header value.
    pub async fn authenticate(&self, authorization: Option<&str>) -> Result<Caller, Error> {
        let token = authorization
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|t| t.starts_with("ck_"))
            .ok_or(Error::Unauthorized)?;
        match self.introspect(token).await? {
            Some(c) if c.kind == "caller" => Ok(c),
            Some(_) => Err(Error::Forbidden("service keys can't render; use a caller key".into())),
            None => Err(Error::Unauthorized),
        }
    }

    /// Drops cached answers for a revoked key (the `key.revoked` event).
    pub fn forget_key(&self, key_id: &str) {
        self.keys.lock().unwrap().retain(|_, (_, c)| c.as_ref().is_none_or(|c| c.id != key_id));
    }

    // --- brands ----------------------------------------------------------------

    fn disk(&self, slug: &str, version: i64) -> Option<PathBuf> {
        (!slug.is_empty() && slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'))
            .then(|| self.cfg.cache_dir.join("brands").join(slug).join(format!("v{version}.json")))
    }

    fn remember(&self, slug: &str, version: i64, doc: &Arc<Value>) {
        self.versions.lock().unwrap().insert((slug.to_string(), version), doc.clone());
        if let Some(p) = self.disk(slug, version).filter(|p| !p.exists()) {
            let _ = std::fs::create_dir_all(p.parent().unwrap());
            let _ = std::fs::write(&p, doc.to_string());
        }
    }

    /// A client's brand: a pinned version, or the latest published one.
    /// Returns the version and the published document (motion/specs/brand.md).
    pub async fn brand(&self, slug: &str, version: Option<i64>) -> Result<(i64, Arc<Value>), Error> {
        match version {
            Some(v) => self.pinned(slug, v).await.map(|d| (v, d)),
            None => self.latest_brand(slug).await,
        }
    }

    async fn pinned(&self, slug: &str, v: i64) -> Result<Arc<Value>, Error> {
        if let Some(d) = self.versions.lock().unwrap().get(&(slug.to_string(), v)) {
            return Ok(d.clone());
        }
        if let Some(d) = self.disk(slug, v).and_then(|p| std::fs::read(p).ok()).and_then(|b| serde_json::from_slice::<Value>(&b).ok()) {
            let d = Arc::new(d);
            self.versions.lock().unwrap().insert((slug.to_string(), v), d.clone());
            return Ok(d);
        }
        let r = self
            .get(&format!("/v1/clients/{slug}/brand/{v}"))
            .send()
            .await
            .map_err(|e| Error::Unavailable(format!("Clients can't be reached and `{slug}` v{v} isn't cached: {e}")))?;
        let d: Arc<Value> = Arc::new(Self::check(r).await?.json().await.map_err(|e| Error::Other(e.to_string()))?);
        self.remember(slug, v, &d);
        Ok(d)
    }

    async fn latest_brand(&self, slug: &str) -> Result<(i64, Arc<Value>), Error> {
        let cached = self.latest.lock().unwrap().get(slug).cloned();
        let mut req = self.get(&format!("/v1/clients/{slug}/brand"));
        if let Some((v, _)) = &cached {
            req = req.header("If-None-Match", format!("\"v{v}\""));
        }
        let r = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                // Clients is down: use what we have (D21).
                return cached.ok_or_else(|| Error::Unavailable(format!("Clients can't be reached and `{slug}` isn't cached: {e}")));
            }
        };
        if r.status().as_u16() == 304 {
            if let Some(c) = &cached {
                return Ok(c.clone());
            }
        }
        let r = match Self::check(r).await {
            Ok(r) => r,
            Err(Error::Unavailable(m)) => return cached.ok_or(Error::Unavailable(m)),
            Err(e) => return Err(e),
        };
        let d: Value = r.json().await.map_err(|e| Error::Other(e.to_string()))?;
        let v = d["version"].as_i64().ok_or_else(|| Error::Other("the brand has no version".into()))?;
        let d = Arc::new(d);
        self.remember(slug, v, &d);
        self.latest.lock().unwrap().insert(slug.to_string(), (v, d.clone()));
        Ok((v, d))
    }

    /// Drops the cached "latest" (the `brand.published` event). Pinned versions stay.
    pub fn forget_latest(&self, slug: &str) {
        self.latest.lock().unwrap().remove(slug);
    }

    /// A client by slug. Cached until a `client.deleted` event; if Clients
    /// is down, the cached answer is used.
    pub async fn client(&self, slug: &str) -> Result<ClientInfo, Error> {
        if let Some(c) = self.clients.lock().unwrap().get(slug) {
            return Ok(c.clone());
        }
        let r = self.get(&format!("/v1/clients/{slug}")).send().await.map_err(|e| Error::Unavailable(e.to_string()))?;
        let c: ClientInfo = Self::check(r).await?.json().await.map_err(|e| Error::Other(e.to_string()))?;
        self.clients.lock().unwrap().insert(slug.to_string(), c.clone());
        Ok(c)
    }

    /// Every client this service key can see.
    pub async fn clients(&self) -> Result<Vec<ClientInfo>, Error> {
        let r = self.get("/v1/clients").send().await.map_err(|e| Error::Unavailable(e.to_string()))?;
        Self::check(r).await?.json().await.map_err(|e| Error::Other(e.to_string()))
    }

    /// A client's library: names → files, for `lib:` references outside the brand.
    pub async fn library(&self, slug: &str) -> Result<Vec<LibraryAsset>, Error> {
        let r = self.get(&format!("/v1/clients/{slug}/assets")).send().await.map_err(|e| Error::Unavailable(e.to_string()))?;
        Self::check(r).await?.json().await.map_err(|e| Error::Other(e.to_string()))
    }

    // --- assets ----------------------------------------------------------------

    fn asset_path(&self, sha: &str) -> Option<PathBuf> {
        (sha.len() == 64 && sha.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()))
            .then(|| self.cfg.cache_dir.join("assets").join(&sha[..2]).join(sha))
    }

    /// A library file by its SHA-256 (from a brand's `assets`). Verified, and
    /// kept on disk until the cache is full.
    pub async fn asset(&self, sha: &str) -> Result<Arc<Vec<u8>>, Error> {
        let p = self.asset_path(sha).ok_or_else(|| Error::Other(format!("`{sha}` isn't a SHA-256")))?;
        if let Ok(b) = tokio::fs::read(&p).await {
            if sha256(&b) == sha {
                return Ok(Arc::new(b));
            }
        }
        let r = self
            .get(&format!("/v1/assets/{sha}"))
            .send()
            .await
            .map_err(|e| Error::Unavailable(format!("Clients can't be reached and asset {sha} isn't cached: {e}")))?;
        let b = Self::check(r).await?.bytes().await.map_err(|e| Error::Unavailable(e.to_string()))?.to_vec();
        if sha256(&b) != sha {
            return Err(Error::Other(format!("asset {sha} arrived corrupted")));
        }
        let dir = p.parent().unwrap();
        let _ = tokio::fs::create_dir_all(dir).await;
        let tmp = dir.join(format!(".{sha}.tmp"));
        if tokio::fs::write(&tmp, &b).await.is_ok() {
            let _ = tokio::fs::rename(&tmp, &p).await;
        }
        self.trim_assets();
        Ok(Arc::new(b))
    }

    /// Keeps the asset cache under its limit, oldest first.
    fn trim_assets(&self) {
        let root = self.cfg.cache_dir.join("assets");
        let mut files: Vec<(std::time::SystemTime, u64, PathBuf)> = vec![];
        for d in std::fs::read_dir(&root).into_iter().flatten().flatten() {
            for f in std::fs::read_dir(d.path()).into_iter().flatten().flatten() {
                if let Ok(m) = f.metadata() {
                    files.push((m.modified().unwrap_or(std::time::UNIX_EPOCH), m.len(), f.path()));
                }
            }
        }
        let mut total: u64 = files.iter().map(|f| f.1).sum();
        files.sort();
        for (_, len, path) in files {
            if total <= self.cfg.asset_cache_bytes {
                break;
            }
            if std::fs::remove_file(&path).is_ok() {
                total -= len;
            }
        }
    }

    // --- events and sign-in --------------------------------------------------------

    /// Events after a cursor, oldest first (at most 100).
    pub async fn events(&self, after: i64) -> Result<Vec<Event>, Error> {
        let r = self.get(&format!("/v1/events?after={after}&limit=100")).send().await.map_err(|e| Error::Unavailable(e.to_string()))?;
        Self::check(r).await?.json().await.map_err(|e| Error::Other(e.to_string()))
    }

    /// Applies the cache side of an event (renderers do their own part, such
    /// as deleting a deleted client's data).
    pub fn apply(&self, e: &Event) {
        match e.kind.as_str() {
            "brand.published" => {
                if let Some(s) = e.slug() {
                    self.forget_latest(s);
                }
            }
            "client.deleted" => {
                if let Some(s) = e.slug() {
                    self.forget_latest(s);
                    self.clients.lock().unwrap().remove(s);
                }
            }
            "key.revoked" => {
                if let Some(id) = e.payload["key"].as_str() {
                    self.forget_key(id);
                }
            }
            _ => {}
        }
    }

    /// Swaps a "Sign in with Clients" code (60 seconds, single use).
    pub async fn handoff(&self, code: &str, app: &str) -> Result<bool, Error> {
        let r = self
            .http
            .post(self.url("/v1/auth/handoff"))
            .bearer_auth(&self.cfg.service_key)
            .json(&json!({ "code": code, "app": app }))
            .send()
            .await;
        let v: Value =
            Self::check(r.map_err(|e| Error::Unavailable(e.to_string()))?).await?.json().await.map_err(|e| Error::Other(e.to_string()))?;
        Ok(v["active"] == true)
    }
}
