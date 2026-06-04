use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::data::guide_assets;
use crate::data::locales;
use crate::util::madness::Madness;

const GUIDE_DIR: &str = "./data/guides";

const KNOWN_GUIDE_SLUGS: &[&str] = &["ddd", "marauder", "documentation", "trainee"];

pub fn is_guide_page(page: &str) -> bool {
    if page == "home" || page == "nav" {
        return false;
    }
    KNOWN_GUIDE_SLUGS.contains(&page) || is_guide(page)
}

pub fn list_guide_slugs() -> Vec<String> {
    let mut slugs = HashSet::new();
    if let Ok(entries) = fs::read_dir(GUIDE_DIR) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                slugs.insert(entry.file_name().to_string_lossy().to_string());
            }
        }
    }
    for slug in KNOWN_GUIDE_SLUGS {
        slugs.insert((*slug).to_string());
    }
    if let Ok(entries) = fs::read_dir("./data/locales") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some((page, _)) = locales::parse_admin_filename(&name) {
                if is_guide_page(&page) {
                    slugs.insert(page);
                }
            }
        }
    }
    let mut list: Vec<String> = slugs.into_iter().collect();
    list.sort();
    list
}

#[derive(Debug, Clone, Serialize)]
pub struct GuideListing {
    pub slug: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    /// `public` (main guides index) or `fc` (FC dashboard).
    pub section: String,
    /// Permission key for FC guides; omitted for public guides.
    pub access: Option<String>,
}

pub fn list_guides() -> Vec<GuideListing> {
    list_guide_slugs()
        .into_iter()
        .filter_map(|slug| guide_listing(&slug).ok())
        .collect()
}

fn guide_listing(slug: &str) -> Result<GuideListing, Madness> {
    let value = locales::read_locale_json(slug, "en").or_else(|_| {
        locales::read_locale_json(slug, &first_locale_for_slug(slug)?)
    })?;

    let body = value
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let title = optional_string_field(&value, "title")
        .filter(|s| !s.is_empty())
        .or_else(|| {
            let from_body = title_from_body(body);
            if from_body.is_empty() {
                None
            } else {
                Some(from_body)
            }
        })
        .unwrap_or_else(|| slug_display_name(slug));

    let section = optional_string_field(&value, "section")
        .unwrap_or_else(|| default_section(slug).to_string());
    let access = optional_string_field(&value, "access").or_else(|| default_access(slug).map(str::to_string));

    Ok(GuideListing {
        slug: slug.to_string(),
        title,
        subtitle: optional_string_field(&value, "subtitle"),
        icon: optional_string_field(&value, "icon"),
        section,
        access,
    })
}

fn first_locale_for_slug(slug: &str) -> Result<String, Madness> {
    let dir = guide_dir(slug)?;
    let Ok(entries) = fs::read_dir(&dir) else {
        return Err(Madness::BadRequest(format!("Guide not found: {}", slug)));
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some((page, locale)) = locales::parse_admin_filename(&name) {
            if page == slug {
                return Ok(locale);
            }
        }
    }
    Err(Madness::BadRequest(format!("Guide not found: {}", slug)))
}

fn optional_string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(|v| v.as_str()).map(str::to_string)
}

fn title_from_body(body: &str) -> String {
    let first = body.lines().next().unwrap_or("").trim();
    if let Some(title) = first.strip_prefix('#') {
        return title.trim().to_string();
    }
    String::new()
}

fn slug_display_name(slug: &str) -> String {
    slug.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn default_section(slug: &str) -> &'static str {
    if matches!(slug, "documentation" | "trainee") {
        "fc"
    } else {
        "public"
    }
}

fn default_access(slug: &str) -> Option<&'static str> {
    match slug {
        "documentation" => Some("waitlist-tag:HQ-FC"),
        "trainee" => Some("waitlist-tag:TRAINEE"),
        _ => None,
    }
}

fn is_guide(page: &str) -> bool {
    locales::validate_page_slug(page).ok().map_or(false, |_| {
        Path::new(&format!("{}/{}", GUIDE_DIR, page)).is_dir()
    })
}

fn legacy_locale_path(page: &str, locale: &str) -> String {
    format!("./data/locales/{}.{}.json", page, locale)
}

pub fn guide_dir(page: &str) -> Result<String, Madness> {
    locales::validate_page_slug(page)?;
    Ok(format!("{}/{}", GUIDE_DIR, page))
}

pub fn locale_file_path(page: &str, locale: &str) -> Result<String, Madness> {
    locales::validate_page_locale(page, locale)?;
    Ok(format!("{}/{}/{}.{}.json", GUIDE_DIR, page, page, locale))
}

pub fn list_admin_filenames() -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = fs::read_dir(GUIDE_DIR) else {
        return legacy_guide_admin_filenames();
    };

    for guide_entry in entries.flatten() {
        if !guide_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let guide_slug = guide_entry.file_name().to_string_lossy().to_string();
        let Ok(locale_entries) = fs::read_dir(guide_entry.path()) else {
            continue;
        };
        for locale_entry in locale_entries.flatten() {
            let name = locale_entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".json") || name.contains(".backup.") {
                continue;
            }
            if locales::parse_admin_filename(&name).is_some() {
                if name.starts_with(&format!("{}.", guide_slug)) {
                    names.push(name);
                }
            }
        }
    }

    for legacy in legacy_guide_admin_filenames() {
        if !names.contains(&legacy) {
            names.push(legacy);
        }
    }

    names.sort();
    names
}

fn legacy_guide_admin_filenames() -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = fs::read_dir("./data/locales") else {
        return names;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".json") || name.contains(".backup.") {
            continue;
        }
        if let Some((page, _locale)) = locales::parse_admin_filename(&name) {
            if is_guide_page(&page) {
                names.push(name);
            }
        }
    }
    names
}

pub fn read_locale(page: &str, locale: &str) -> Result<String, Madness> {
    let path = locale_file_path(page, locale)?;
    if Path::new(&path).exists() {
        return fs::read_to_string(&path)
            .map_err(|e| Madness::BadRequest(format!("Failed to read locale file: {}", e)));
    }

    let legacy = legacy_locale_path(page, locale);
    if Path::new(&legacy).exists() {
        return fs::read_to_string(&legacy)
            .map_err(|e| Madness::BadRequest(format!("Failed to read locale file: {}", e)));
    }

    Err(Madness::BadRequest(format!(
        "Locale file not found: {}.{}.json",
        page, locale
    )))
}

pub fn save_locale(page: &str, locale: &str, content: &str) -> Result<(), Madness> {
    let path = locale_file_path(page, locale)?;
    locales::parse_locale_json(content)?;

    let dir = guide_dir(page)?;
    fs::create_dir_all(&dir)
        .map_err(|e| Madness::BadRequest(format!("Failed to create guide directory: {}", e)))?;

    create_backup(&path)?;

    let temp_path = format!("{}.tmp", path);
    fs::write(&temp_path, content)
        .map_err(|e| Madness::BadRequest(format!("Failed to write temp file: {}", e)))?;
    fs::rename(&temp_path, &path)
        .map_err(|e| Madness::BadRequest(format!("Failed to save locale file: {}", e)))?;

    Ok(())
}

pub fn delete_locale(page: &str, locale: &str) -> Result<(), Madness> {
    let path = locale_file_path(page, locale)?;
    if !Path::new(&path).exists() {
        return Err(Madness::BadRequest(format!(
            "Locale file not found: {}.{}.json",
            page, locale
        )));
    }
    create_backup(&path)?;
    fs::remove_file(&path)
        .map_err(|e| Madness::BadRequest(format!("Failed to delete locale file: {}", e)))
}

pub fn file_info_for_admin(filename: &str) -> Option<(u64, bool)> {
    let (page, locale) = locales::parse_admin_filename(filename)?;
    if let Ok(path) = locale_file_path(&page, &locale) {
        if Path::new(&path).exists() {
            let metadata = fs::metadata(&path).ok()?;
            return Some((metadata.len(), false));
        }
    }
    let legacy = legacy_locale_path(&page, &locale);
    if !Path::new(&legacy).exists() {
        return Some((0, false));
    }
    let metadata = fs::metadata(&legacy).ok()?;
    Some((metadata.len(), false))
}

fn validate_asset_filename(filename: &str) -> Result<(), Madness> {
    if filename.is_empty()
        || filename.contains("..")
        || filename.contains('/')
        || filename.contains('\\')
    {
        return Err(Madness::BadRequest("Invalid asset filename".to_string()));
    }
    let extension = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| Madness::BadRequest("Asset must have a file extension".to_string()))?;
    if !matches!(
        extension.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg"
    ) {
        return Err(Madness::BadRequest(
            "Unsupported guide asset type".to_string(),
        ));
    }
    Ok(())
}

pub fn read_asset(page: &str, filename: &str) -> Result<Vec<u8>, Madness> {
    validate_asset_filename(filename)?;
    let dir = guide_dir(page)?;
    let path = format!("{}/{}", dir, filename);
    if !Path::new(&path).exists() {
        return Err(Madness::BadRequest(format!(
            "Guide asset not found: {}",
            filename
        )));
    }
    fs::read(&path).map_err(|e| Madness::BadRequest(format!("Failed to read asset: {}", e)))
}

pub fn asset_mime_type(filename: &str) -> Result<&'static str, Madness> {
    validate_asset_filename(filename)?;
    let extension = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    Ok(match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct GuideAssetInfo {
    pub name: String,
    pub size: u64,
}

const MAX_ASSET_BYTES: usize = 10 * 1024 * 1024;
const OPTIMIZE_MIN_BYTES: usize = 1024 * 1024;

pub fn list_assets(page: &str) -> Result<Vec<GuideAssetInfo>, Madness> {
    let dir = guide_dir(page)?;
    let path = Path::new(&dir);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut assets = Vec::new();
    for entry in fs::read_dir(&dir)
        .map_err(|e| Madness::BadRequest(format!("Failed to read guide directory: {}", e)))?
        .flatten()
    {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(".backup.") || validate_asset_filename(&name).is_err() {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        assets.push(GuideAssetInfo { name, size });
    }
    assets.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(assets)
}

pub fn save_asset(page: &str, filename: &str, data: &[u8]) -> Result<(String, u64), Madness> {
    validate_asset_filename(filename)?;
    if data.is_empty() {
        return Err(Madness::BadRequest("Asset must not be empty".to_string()));
    }
    if data.len() > MAX_ASSET_BYTES {
        return Err(Madness::BadRequest(
            "Asset exceeds 10 MiB upload limit".to_string(),
        ));
    }

    let original_len = data.len();
    let optimized = if original_len > OPTIMIZE_MIN_BYTES {
        match guide_assets::optimize_guide_image(filename, data) {
            Ok(optimized) => {
                if optimized.bytes.len() < original_len || optimized.filename != filename {
                    eprintln!(
                        "guide asset {} -> {} ({} -> {} bytes)",
                        filename,
                        optimized.filename,
                        original_len,
                        optimized.bytes.len()
                    );
                }
                optimized
            }
            Err(reason) => {
                if reason != "not optimized" {
                    eprintln!(
                        "guide asset {}: optimization skipped ({})",
                        filename, reason
                    );
                }
                guide_assets::OptimizedGuideImage {
                    bytes: data.to_vec(),
                    filename: filename.to_string(),
                }
            }
        }
    } else {
        guide_assets::OptimizedGuideImage {
            bytes: data.to_vec(),
            filename: filename.to_string(),
        }
    };

    let data = optimized.bytes;
    let save_filename = optimized.filename;
    if data.is_empty() {
        return Err(Madness::BadRequest("Asset must not be empty".to_string()));
    }
    if data.len() > MAX_ASSET_BYTES {
        return Err(Madness::BadRequest(
            "Asset still exceeds 10 MiB after compression; use a smaller image".to_string(),
        ));
    }

    let dir = guide_dir(page)?;
    fs::create_dir_all(&dir)
        .map_err(|e| Madness::BadRequest(format!("Failed to create guide directory: {}", e)))?;

    let path = format!("{}/{}", dir, save_filename);
    create_backup(&path)?;

    let temp_path = format!("{}.tmp", path);
    fs::write(&temp_path, &data)
        .map_err(|e| Madness::BadRequest(format!("Failed to write asset: {}", e)))?;
    fs::rename(&temp_path, &path)
        .map_err(|e| Madness::BadRequest(format!("Failed to save asset: {}", e)))?;
    Ok((save_filename, data.len() as u64))
}

pub fn delete_asset(page: &str, filename: &str) -> Result<(), Madness> {
    validate_asset_filename(filename)?;
    let path = format!("{}/{}", guide_dir(page)?, filename);
    if !Path::new(&path).exists() {
        return Err(Madness::BadRequest(format!(
            "Guide asset not found: {}",
            filename
        )));
    }
    fs::remove_file(&path)
        .map_err(|e| Madness::BadRequest(format!("Failed to delete asset: {}", e)))
}

fn create_backup(path: &str) -> Result<(), Madness> {
    if !Path::new(path).exists() {
        return Ok(());
    }
    let parent = Path::new(path)
        .parent()
        .and_then(|p| p.to_str())
        .ok_or_else(|| Madness::BadRequest("Invalid guide path".to_string()))?;
    let file_name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Madness::BadRequest("Invalid guide path".to_string()))?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let backup_path = format!("{}/{}.backup.{}", parent, file_name, timestamp);
    fs::copy(path, &backup_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create backup: {}", e)))?;

    let backup_prefix = format!("{}.backup.", file_name);
    let mut backups: Vec<_> = fs::read_dir(parent)
        .map_err(|e| Madness::BadRequest(format!("Failed to read guide directory: {}", e)))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&backup_prefix) {
                if let Some(timestamp_str) = name.strip_prefix(&backup_prefix) {
                    if let Ok(ts) = timestamp_str.parse::<u64>() {
                        return Some((entry.path(), ts));
                    }
                }
            }
            None
        })
        .collect();

    backups.sort_by(|a, b| b.1.cmp(&a.1));
    for (backup_path, _) in backups.into_iter().skip(5) {
        let _ = fs::remove_file(backup_path);
    }
    Ok(())
}
