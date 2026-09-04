//! Runtime font resolution via [`fontdb`]: scan the operating system's font
//! database and pick a face by family name, falling back to the built-in
//! (embedded `include_bytes!`) assets if no matching font is installed.
//!
//! - [`en_font`]  — Latin font for the default English UI (small, low memory).
//! - [`zh_font`]  — CJK font, loaded only when the user switches to Chinese.
//!
//! Both resolve once and cache the result, so a failed system lookup does not
//! panic — it silently degrades to the embedded asset.

use fontdb::{Database, Family, Query, Source, Stretch, Style, Weight};
use ply_engine::prelude::FontAsset;
use std::sync::OnceLock;

/// Preferred Latin family names for the English UI, in order of preference.
const EN_FAMILIES: &[&str] = &["Segoe UI", "Arial", "Helvetica", "DejaVu Sans"];

/// Embedded assets (CJK 用子集,见 tools/subset_font.py).
static EN_EMBEDDED: FontAsset = FontAsset::Bytes {
    file_name: "lexend.ttf",
    data: include_bytes!("../assets/fonts/lexend.ttf"),
};

static ZH_EMBEDDED: FontAsset = FontAsset::Bytes {
    file_name: "LXGWWenKai-Medium.ttf",
    data: include_bytes!("../assets/fonts/LXGWWenKai-Medium.ttf"),
};

static EN_FONT: OnceLock<&'static FontAsset> = OnceLock::new();
static ZH_FONT: OnceLock<&'static FontAsset> = OnceLock::new();

/// Latin font for the default English UI. Looks up a system family, falls back
/// to the embedded lexend asset. Loaded at startup (small, low memory).
pub fn en_font() -> &'static FontAsset {
    *EN_FONT.get_or_init(|| find_system_font(EN_FAMILIES).unwrap_or(&EN_EMBEDDED))
}

/// CJK font: embedded subset FIRST (25MB full font costs ~140MB parse heap;
/// subset is 1.7MB; system CJK fonts are equally huge). Consistent look on all
/// platforms + half the memory. See `tools/subset_font.py`.
pub fn zh_font() -> &'static FontAsset {
    *ZH_FONT.get_or_init(|| &ZH_EMBEDDED)
}

/// Scans the system font database once and returns the first matching face as a
/// leaked `FontAsset::Bytes`, so it can back the `'static` asset at runtime.
fn find_system_font(families: &[&str]) -> Option<&'static FontAsset> {
    let mut db = Database::new();
    db.load_system_fonts();

    for family in families {
        let query = Query {
            families: &[Family::Name(family)],
            weight: Weight::NORMAL,
            stretch: Stretch::Normal,
            style: Style::Normal,
        };
        let Some(id) = db.query(&query) else { continue };
        let Some(face) = db.face(id) else { continue };
        let asset = match &face.source {
            Source::File(path) => {
                let bytes = std::fs::read(path).ok()?;
                let data: &'static [u8] = Box::leak(bytes.into_boxed_slice());
                let file_name: &'static str = Box::leak(path.to_string_lossy().into_owned().into_boxed_str());
                FontAsset::Bytes { file_name, data }
            }
            Source::Binary(data) => {
                let arc: &'static std::sync::Arc<dyn AsRef<[u8]> + Send + Sync> =
                    Box::leak(Box::new(data.clone()));
                FontAsset::Bytes {
                    file_name: "system-font",
                    data: arc.as_ref().as_ref(),
                }
            }
            Source::SharedFile(_, data) => {
                let arc: &'static std::sync::Arc<dyn AsRef<[u8]> + Send + Sync> =
                    Box::leak(Box::new(data.clone()));
                FontAsset::Bytes {
                    file_name: "system-font",
                    data: arc.as_ref().as_ref(),
                }
            }
        };
        return Some(Box::leak(Box::new(asset)));
    }
    None
}
