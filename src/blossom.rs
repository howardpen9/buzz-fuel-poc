//! Blossom BUD-02 upload client (kind 24242 auth) for the same Buzz relay.

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use nostr::{EventBuilder, JsonUtil, Keys, Kind, Tag, Timestamp};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};

const MIME_PNG: &str = "image/png";
const AUTH_EXPIRY_SECS: u64 = 600;

/// Minimal BlobDescriptor fields we need after `PUT /upload`.
#[derive(Debug, Clone, Deserialize)]
pub struct BlobDescriptor {
    pub url: String,
    pub sha256: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub mime_type: String,
    #[serde(default)]
    pub dim: Option<String>,
}

/// Upload result plus NIP-92 imeta tag vector for `build_message`.
#[derive(Debug, Clone)]
pub struct UploadedMedia {
    pub url: String,
    pub imeta_tag: Vec<String>,
}

/// Map `ws://` / `wss://` relay URL to HTTP origin for Blossom.
pub fn http_base_from_relay(relay_url: &str) -> Result<String> {
    let mut u = url::Url::parse(relay_url).context("parse BUZZ_RELAY_URL")?;
    match u.scheme() {
        "ws" => {
            u.set_scheme("http")
                .map_err(|_| anyhow!("cannot map ws → http"))?;
        }
        "wss" => {
            u.set_scheme("https")
                .map_err(|_| anyhow!("cannot map wss → https"))?;
        }
        "http" | "https" => {}
        other => bail!("unsupported relay URL scheme: {other}"),
    }
    // Strip path/query so we land on origin (relay media is at {origin}/upload).
    u.set_path("");
    u.set_query(None);
    u.set_fragment(None);
    let mut s = u.to_string();
    if s.ends_with('/') {
        s.pop();
    }
    Ok(s)
}

/// `host[:port]` for Blossom `server` tag (matches CLI/Desktop authority shape).
pub fn server_authority(http_base: &str) -> Option<String> {
    let u = url::Url::parse(http_base).ok()?;
    let host = u.host_str()?;
    match u.port() {
        Some(port) => Some(format!("{host}:{port}")),
        None => Some(host.to_string()),
    }
}

fn sign_blossom_upload_auth(keys: &Keys, sha256: &str, http_base: &str) -> Result<String> {
    let now = Timestamp::now().as_secs();
    let mut tags = vec![
        Tag::parse(["t", "upload"])?,
        Tag::parse(["x", sha256])?,
        Tag::parse(["expiration", &(now + AUTH_EXPIRY_SECS).to_string()])?,
    ];
    if let Some(domain) = server_authority(http_base) {
        tags.push(Tag::parse(["server", &domain])?);
    }
    let event = EventBuilder::new(Kind::from(24242), "Upload buzz-fuel QR")
        .tags(tags)
        .sign_with_keys(keys)?;
    Ok(format!(
        "Nostr {}",
        URL_SAFE_NO_PAD.encode(event.as_json().as_bytes())
    ))
}

/// Build NIP-92 imeta tag from upload metadata (relay-local `/media/` URL).
pub fn build_imeta_tag(
    url: &str,
    mime_type: &str,
    sha256: &str,
    size: u64,
    dim: Option<&str>,
) -> Vec<String> {
    let mut tag = vec![
        "imeta".to_string(),
        format!("url {url}"),
        format!("m {mime_type}"),
        format!("x {sha256}"),
        format!("size {size}"),
    ];
    if let Some(d) = dim {
        if !d.is_empty() {
            tag.push(format!("dim {d}"));
        }
    }
    tag
}

/// PUT PNG bytes to `{http_base}/upload` with kind 24242 Blossom auth.
///
/// `auth_tag_json` is the optional NIP-OA `x-auth-tag` header (owner-attested bots).
pub async fn upload_png(
    http_base: &str,
    keys: &Keys,
    png_bytes: &[u8],
    dim: Option<&str>,
    auth_tag_json: Option<&str>,
) -> Result<UploadedMedia> {
    let sha256 = hex::encode(Sha256::digest(png_bytes));
    let auth_header = sign_blossom_upload_auth(keys, &sha256, http_base)?;
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .context("http client for blossom upload")?;

    let mut req = client
        .put(format!("{http_base}/upload"))
        .header("Authorization", &auth_header)
        .header("Content-Type", MIME_PNG)
        .header("X-SHA-256", &sha256)
        .body(png_bytes.to_vec());
    if let Some(tag) = auth_tag_json {
        req = req.header("x-auth-tag", tag);
    }

    let resp = req.send().await.context("blossom PUT /upload failed")?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        // Legacy alias used by older relays.
        if status.as_u16() == 404 || status.as_u16() == 405 {
            return upload_png_legacy(http_base, keys, png_bytes, &sha256, dim, auth_tag_json)
                .await;
        }
        bail!("blossom upload HTTP {status}: {body}");
    }

    let desc: BlobDescriptor =
        serde_json::from_str(&body).context("decode BlobDescriptor from upload response")?;
    Ok(uploaded_from_descriptor(desc, dim))
}

async fn upload_png_legacy(
    http_base: &str,
    keys: &Keys,
    png_bytes: &[u8],
    sha256: &str,
    dim: Option<&str>,
    auth_tag_json: Option<&str>,
) -> Result<UploadedMedia> {
    let auth_header = sign_blossom_upload_auth(keys, sha256, http_base)?;
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    let mut req = client
        .put(format!("{http_base}/media/upload"))
        .header("Authorization", &auth_header)
        .header("Content-Type", MIME_PNG)
        .header("X-SHA-256", sha256)
        .body(png_bytes.to_vec());
    if let Some(tag) = auth_tag_json {
        req = req.header("x-auth-tag", tag);
    }
    let resp = req.send().await.context("blossom PUT /media/upload failed")?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        bail!("blossom legacy upload HTTP {status}: {body}");
    }
    let desc: BlobDescriptor =
        serde_json::from_str(&body).context("decode BlobDescriptor from legacy upload")?;
    Ok(uploaded_from_descriptor(desc, dim))
}

fn uploaded_from_descriptor(desc: BlobDescriptor, fallback_dim: Option<&str>) -> UploadedMedia {
    let dim = desc.dim.as_deref().or(fallback_dim);
    let imeta_tag = build_imeta_tag(
        &desc.url,
        &desc.mime_type,
        &desc.sha256,
        desc.size,
        dim,
    );
    UploadedMedia {
        url: desc.url,
        imeta_tag,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_base_maps_ws_to_http() {
        assert_eq!(
            http_base_from_relay("ws://localhost:3000").unwrap(),
            "http://localhost:3000"
        );
        assert_eq!(
            http_base_from_relay("wss://relay.example.com/").unwrap(),
            "https://relay.example.com"
        );
        assert_eq!(
            http_base_from_relay("http://localhost:3000/path").unwrap(),
            "http://localhost:3000"
        );
    }

    #[test]
    fn server_authority_keeps_explicit_port() {
        assert_eq!(
            server_authority("http://localhost:3000").as_deref(),
            Some("localhost:3000")
        );
        assert_eq!(
            server_authority("https://relay.example.com").as_deref(),
            Some("relay.example.com")
        );
    }

    #[test]
    fn imeta_tag_shape_matches_nip92() {
        let tag = build_imeta_tag(
            "http://localhost:3000/media/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
            "image/png",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            1234,
            Some("320x320"),
        );
        assert_eq!(tag[0], "imeta");
        assert!(tag.iter().any(|p| p.starts_with("url ")));
        assert!(tag.iter().any(|p| p == "m image/png"));
        assert!(tag.iter().any(|p| p.starts_with("x ")));
        assert!(tag.iter().any(|p| p == "size 1234"));
        assert!(tag.iter().any(|p| p == "dim 320x320"));
    }
}

#[cfg(test)]
mod live_smoke {
    use super::*;
    use crate::qr;

    /// Local relay only: `cargo test blossom_live_upload -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn blossom_live_upload() {
        let keys = Keys::generate();
        let png = qr::encode_qr_png("https://t.me/MakeReel_xyz_bot?start=fuel_smoke").unwrap();
        let dim = format!("{}x{}", png.width, png.height);
        let media = upload_png(
            "http://localhost:3000",
            &keys,
            &png.bytes,
            Some(&dim),
            None,
        )
        .await
        .expect("upload to local relay");
        assert!(media.url.contains("/media/"));
        assert!(media.imeta_tag.iter().any(|p| p.starts_with("url ")));
        println!("uploaded {}", media.url);
    }
}
