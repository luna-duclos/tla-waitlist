use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::data::guides;
use crate::util::madness::Madness;

const LOCALE_DIR: &str = "./data/locales";
pub const LANGUAGE_LABELS_FILENAME: &str = "languages.json";
const LANGUAGE_LABELS_PATH: &str = "./data/locales/languages.json";

pub fn validate_page_slug(page: &str) -> Result<(), Madness> {
    if page.is_empty() || page.len() > 32 {
        return Err(Madness::BadRequest(
            "Page slug must be 1–32 characters".to_string(),
        ));
    }
    let mut chars = page.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_lowercase() {
        return Err(Madness::BadRequest(
            "Page slug must start with a lowercase letter".to_string(),
        ));
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
        return Err(Madness::BadRequest(
            "Page slug may only use lowercase letters, numbers, hyphens, and underscores".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_locale_code(locale: &str) -> Result<(), Madness> {
    if locale.len() != 2 || !locale.chars().all(|c| c.is_ascii_lowercase()) {
        return Err(Madness::BadRequest(
            "Locale must be a 2-letter lowercase code (e.g. en, de, fr)".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_page_locale(page: &str, locale: &str) -> Result<(), Madness> {
    validate_page_slug(page)?;
    validate_locale_code(locale)?;
    Ok(())
}

pub fn locale_file_path(page: &str, locale: &str) -> Result<String, Madness> {
    if guides::is_guide_page(page) {
        return guides::locale_file_path(page, locale);
    }
    validate_page_locale(page, locale)?;
    Ok(format!("{}/{}.{}.json", LOCALE_DIR, page, locale))
}

pub fn admin_filename(page: &str, locale: &str) -> Result<String, Madness> {
    validate_page_locale(page, locale)?;
    Ok(format!("{}.{}.json", page, locale))
}

/// Admin data-files use flat names like `home.en.json`.
pub fn parse_admin_filename(filename: &str) -> Option<(String, String)> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return None;
    }
    let stem = filename.strip_suffix(".json")?;
    let mut parts = stem.splitn(2, '.');
    let page = parts.next()?.to_string();
    let locale = parts.next()?.to_string();
    if parts.next().is_some() {
        return None;
    }
    validate_page_locale(&page, &locale).ok()?;
    Some((page, locale))
}

pub fn is_locale_admin_filename(filename: &str) -> bool {
    parse_admin_filename(filename).is_some()
}

pub fn list_admin_filenames() -> Vec<String> {
    let mut names = guides::list_admin_filenames();

    let Ok(entries) = fs::read_dir(LOCALE_DIR) else {
        names.sort();
        return names;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".json") || name.contains(".backup.") {
            continue;
        }
        if let Some((page, _locale)) = parse_admin_filename(&name) {
            if guides::is_guide_page(&page) {
                continue;
            }
            names.push(name);
        }
    }

    names.sort();
    names
}

pub fn locale_file_exists(page: &str, locale: &str) -> bool {
    locale_file_path(page, locale)
        .ok()
        .map(|path| Path::new(&path).exists())
        .unwrap_or(false)
}

/// Locales that have both `home.{locale}.json` and `nav.{locale}.json`.
pub fn list_valid_languages() -> Vec<String> {
    let home = locales_for_page("home");
    let nav = locales_for_page("nav");
    let mut languages: Vec<String> = home.intersection(&nav).cloned().collect();
    languages.sort();
    languages
}

pub fn is_language_labels_filename(filename: &str) -> bool {
    filename == LANGUAGE_LABELS_FILENAME
}

pub fn read_language_labels_json() -> Result<String, Madness> {
    let path = Path::new(LANGUAGE_LABELS_PATH);
    if !path.exists() {
        return Ok("{}".to_string());
    }
    fs::read_to_string(LANGUAGE_LABELS_PATH)
        .map_err(|e| Madness::BadRequest(format!("Failed to read language labels: {}", e)))
}

pub fn read_language_labels_map() -> Result<HashMap<String, String>, Madness> {
    let content = read_language_labels_json()?;
    let value: Value = serde_json::from_str(&content)
        .map_err(|e| Madness::BadRequest(format!("Invalid language labels JSON: {}", e)))?;
    parse_language_labels(&value)
}

fn parse_language_labels(value: &Value) -> Result<HashMap<String, String>, Madness> {
    let obj = value.as_object().ok_or_else(|| {
        Madness::BadRequest("Language labels must be a JSON object".to_string())
    })?;
    let mut map = HashMap::new();
    for (key, val) in obj {
        validate_locale_code(key)?;
        let label = val.as_str().ok_or_else(|| {
            Madness::BadRequest(format!("Language label for {} must be a string", key))
        })?;
        if label.is_empty() {
            return Err(Madness::BadRequest(format!(
                "Language label for {} must not be empty",
                key
            )));
        }
        map.insert(key.clone(), label.to_string());
    }
    Ok(map)
}

pub fn validate_language_labels_json(content: &str) -> Result<(), Madness> {
    let value: Value = serde_json::from_str(content)
        .map_err(|e| Madness::BadRequest(format!("Invalid JSON: {}", e)))?;
    parse_language_labels(&value)?;
    Ok(())
}

pub fn save_language_labels(content: &str) -> Result<(), Madness> {
    validate_language_labels_json(content)?;
    let path = Path::new(LANGUAGE_LABELS_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            Madness::BadRequest(format!("Failed to create locale directory: {}", e))
        })?;
    }
    fs::write(LANGUAGE_LABELS_PATH, content)
        .map_err(|e| Madness::BadRequest(format!("Failed to save language labels: {}", e)))
}

pub fn file_info_language_labels() -> Option<(u64, bool)> {
    let path = Path::new(LANGUAGE_LABELS_PATH);
    if !path.exists() {
        return None;
    }
    let metadata = fs::metadata(LANGUAGE_LABELS_PATH).ok()?;
    Some((metadata.len(), false))
}

/// Labels from `languages.json`, with uppercase code fallback for each valid language.
pub fn language_labels_for_languages(languages: &[String]) -> HashMap<String, String> {
    let file_labels = read_language_labels_map().unwrap_or_default();
    let mut result = HashMap::new();
    for code in languages {
        let label = file_labels
            .get(code)
            .cloned()
            .unwrap_or_else(|| code.to_uppercase());
        result.insert(code.clone(), label);
    }
    result
}

fn locales_for_page(page: &str) -> HashSet<String> {
    let mut locales = HashSet::new();
    let Ok(entries) = fs::read_dir(LOCALE_DIR) else {
        return locales;
    };
    let prefix = format!("{}.", page);
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with(&prefix) || !name.ends_with(".json") || name.contains(".backup.") {
            continue;
        }
        if let Some((parsed_page, locale)) = parse_admin_filename(&name) {
            if parsed_page == page {
                locales.insert(locale);
            }
        }
    }
    locales
}

pub fn read_locale(page: &str, locale: &str) -> Result<String, Madness> {
    if guides::is_guide_page(page) {
        return guides::read_locale(page, locale);
    }
    let path = locale_file_path(page, locale)?;
    if !Path::new(&path).exists() {
        return Err(Madness::BadRequest(format!(
            "Locale file not found: {}.{}.json",
            page, locale
        )));
    }
    fs::read_to_string(&path)
        .map_err(|e| Madness::BadRequest(format!("Failed to read locale file: {}", e)))
}

pub fn read_locale_json(page: &str, locale: &str) -> Result<Value, Madness> {
    let content = read_locale(page, locale)?;
    parse_locale_json(&content)
}

pub fn parse_locale_json(content: &str) -> Result<Value, Madness> {
    let value: Value = serde_json::from_str(content)
        .map_err(|e| Madness::BadRequest(format!("Invalid JSON: {}", e)))?;
    validate_locale_object(&value)?;
    Ok(value)
}

fn validate_locale_object(value: &Value) -> Result<(), Madness> {
    let obj = value
        .as_object()
        .ok_or_else(|| Madness::BadRequest("Locale file must be a JSON object".to_string()))?;
    if obj.is_empty() {
        return Err(Madness::BadRequest(
            "Locale file must not be empty".to_string(),
        ));
    }
    for (key, val) in obj {
        if key.is_empty() {
            return Err(Madness::BadRequest("Keys must not be empty".to_string()));
        }
        if !val.is_string() {
            return Err(Madness::BadRequest(format!(
                "Key \"{}\" must have a string value",
                key
            )));
        }
    }
    Ok(())
}

pub fn save_locale_from_admin_filename(filename: &str, content: &str) -> Result<(), Madness> {
    let (page, locale) = parse_admin_filename(filename)
        .ok_or_else(|| Madness::BadRequest("Invalid locale filename".to_string()))?;
    save_locale(&page, &locale, content)
}

pub fn save_locale(page: &str, locale: &str, content: &str) -> Result<(), Madness> {
    if guides::is_guide_page(page) {
        return guides::save_locale(page, locale, content);
    }
    let path = locale_file_path(page, locale)?;
    parse_locale_json(content)?;

    fs::create_dir_all(LOCALE_DIR)
        .map_err(|e| Madness::BadRequest(format!("Failed to create locale directory: {}", e)))?;

    create_backup(&path)?;

    let temp_path = format!("{}.tmp", path);
    fs::write(&temp_path, content)
        .map_err(|e| Madness::BadRequest(format!("Failed to write temp file: {}", e)))?;
    fs::rename(&temp_path, &path)
        .map_err(|e| Madness::BadRequest(format!("Failed to save locale file: {}", e)))?;

    Ok(())
}

pub fn delete_locale_from_admin_filename(filename: &str) -> Result<(), Madness> {
    let (page, locale) = parse_admin_filename(filename)
        .ok_or_else(|| Madness::BadRequest("Invalid locale filename".to_string()))?;
    delete_locale(&page, &locale)
}

pub fn delete_locale(page: &str, locale: &str) -> Result<(), Madness> {
    if guides::is_guide_page(page) {
        return guides::delete_locale(page, locale);
    }
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

fn create_backup(path: &str) -> Result<(), Madness> {
    if !Path::new(path).exists() {
        return Ok(());
    }
    let file_name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Madness::BadRequest("Invalid locale path".to_string()))?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let backup_path = format!("{}/{}.backup.{}", LOCALE_DIR, file_name, timestamp);
    fs::copy(path, &backup_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create backup: {}", e)))?;

    let backup_prefix = format!("{}.backup.", file_name);
    let mut backups: Vec<_> = fs::read_dir(LOCALE_DIR)
        .map_err(|e| Madness::BadRequest(format!("Failed to read locale directory: {}", e)))?
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
    for (backup_path, _) in backups.into_iter().skip(3) {
        let _ = fs::remove_file(backup_path);
    }
    Ok(())
}

pub fn file_info_for_admin(filename: &str) -> Option<(u64, bool)> {
    let (page, locale) = parse_admin_filename(filename)?;
    if guides::is_guide_page(&page) {
        return guides::file_info_for_admin(filename);
    }
    let path = locale_file_path(&page, &locale).ok()?;
    if !Path::new(&path).exists() {
        return Some((0, false));
    }
    let metadata = fs::metadata(&path).ok()?;
    Some((metadata.len(), false))
}
