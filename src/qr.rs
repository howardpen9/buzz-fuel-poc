//! QR PNG encoding for public Telegram fuel deep links only.

use anyhow::{Context, Result};
use image::ImageFormat;
use qrcode::QrCode;
use std::io::Cursor;

/// Demo-friendly module scale: ~256–512 px on laptop screens.
const MIN_DIMENSION: u32 = 320;

/// QR payload is **only** the public deep link. Never intent IDs, secrets, or payment rails.
pub fn qr_payload(telegram_url: &str) -> &str {
    telegram_url
}

/// Encoded PNG for `telegram_url` plus pixel dimensions for imeta `dim`.
#[derive(Debug, Clone)]
pub struct QrPng {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Encode `payload` (must be the public `t.me` deep link) as a scannable PNG.
pub fn encode_qr_png(payload: &str) -> Result<QrPng> {
    let code = QrCode::new(payload.as_bytes()).context("QR encode failed")?;
    let img = code
        .render::<image::Luma<u8>>()
        .min_dimensions(MIN_DIMENSION, MIN_DIMENSION)
        .quiet_zone(true)
        .build();
    let (width, height) = img.dimensions();
    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .context("PNG encode failed")?;
    Ok(QrPng {
        bytes,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qr_payload_is_public_telegram_url_only() {
        let url = "https://t.me/MakeReel_xyz_bot?start=fuel_opaque_token";
        assert_eq!(qr_payload(url), url);
        // Explicit: helpers must not inject operator IDs into the QR string.
        assert!(!qr_payload(url).contains("intent:"));
        assert!(!qr_payload(url).contains("USDC"));
        assert!(!qr_payload(url).contains("x402"));
    }

    #[test]
    fn encode_produces_png_magic() {
        let url = "https://t.me/MakeReel_xyz_bot?start=fuel_abc";
        let png = encode_qr_png(qr_payload(url)).expect("encode");
        assert!(
            png.bytes.starts_with(&[0x89, b'P', b'N', b'G']),
            "expected PNG signature"
        );
        assert!(png.width >= MIN_DIMENSION);
        assert!(png.height >= MIN_DIMENSION);
        assert!(png.bytes.len() > 100);
    }

    #[test]
    fn different_urls_yield_different_pngs() {
        let a = encode_qr_png("https://t.me/MakeReel_xyz_bot?start=fuel_a").unwrap();
        let b = encode_qr_png("https://t.me/MakeReel_xyz_bot?start=fuel_b").unwrap();
        assert_ne!(a.bytes, b.bytes);
    }
}
