use rocket::data::{Data, ToByteUnit};
use rocket::serde::json::Json;
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::{
    core::auth::AuthenticatedAccount,
    data::{guides, locales},
    util::madness::Madness,
};

const DATA_DIR: &str = "./data";

#[derive(Debug, Serialize)]
struct DataFileInfo {
    name: String,
    file_type: String,
    requires_reload: bool,
    size: u64,
    is_guide: bool,
}

#[derive(Debug, Serialize)]
struct DataFilesListResponse {
    files: Vec<DataFileInfo>,
}

fn get_file_info(filename: &str) -> Option<DataFileInfo> {
    let file_path = format!("{}/{}", DATA_DIR, filename);
    let path = Path::new(&file_path);
    
    if !path.exists() {
        return None;
    }
    
    let metadata = fs::metadata(&file_path).ok()?;
    let requires_reload = matches!(
        filename,
        "skills.yaml" | "categories.yaml" | "modules.yaml" | "tags.yaml" | "fits.dat"
    );
    
    let file_type = if filename.ends_with(".yaml") {
        "YAML"
    } else if filename.ends_with(".dat") {
        "DAT"
    } else {
        "Unknown"
    }.to_string();
    
    Some(DataFileInfo {
        name: filename.to_string(),
        file_type,
        requires_reload,
        size: metadata.len(),
        is_guide: false,
    })
}

#[get("/api/admin/data-files")]
fn list_data_files(account: AuthenticatedAccount) -> Result<Json<DataFilesListResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;
    
    let mut files = Vec::new();
    
    // List all files in data directory
    let entries = fs::read_dir(DATA_DIR)
        .map_err(|e| Madness::BadRequest(format!("Failed to read data directory: {}", e)))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| Madness::BadRequest(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();
        
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            // Skip backup files
            if filename.contains(".backup.") {
                continue;
            }
            
            // Only include known editable files
            if matches!(
                filename,
                "skills.yaml" | "categories.yaml" | "modules.yaml" | "tags.yaml" | 
                "fits.dat" | "fitnotes.yaml" | "skillplan.yaml"
            ) {
                if let Some(info) = get_file_info(filename) {
                    files.push(info);
                }
            }
        }
    }
    
    if let Some((size, requires_reload)) = locales::file_info_language_labels() {
        files.push(DataFileInfo {
            name: locales::LANGUAGE_LABELS_FILENAME.to_string(),
            file_type: "Language labels".to_string(),
            requires_reload,
            size,
            is_guide: false,
        });
    }

    for filename in locales::list_admin_filenames() {
        let (size, requires_reload) = locales::file_info_for_admin(&filename).unwrap_or((0, false));
        let is_guide = locales::parse_admin_filename(&filename)
            .map(|(page, _)| guides::is_guide_page(&page))
            .unwrap_or(false);
        files.push(DataFileInfo {
            name: filename,
            file_type: if is_guide {
                "Guide JSON".to_string()
            } else {
                "Locale JSON".to_string()
            },
            requires_reload,
            size,
            is_guide,
        });
    }

    files.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(Json(DataFilesListResponse { files }))
}

#[get("/api/admin/data-files/<filename>")]
fn get_data_file(account: AuthenticatedAccount, filename: String) -> Result<String, Madness> {
    account.require_access("commanders-manage:admin")?;
    
    // Validate filename to prevent directory traversal
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err(Madness::BadRequest("Invalid filename".to_string()));
    }
    
    if let Some((page, locale)) = locales::parse_admin_filename(&filename) {
        return locales::read_locale(&page, &locale);
    }

    if locales::is_language_labels_filename(&filename) {
        return locales::read_language_labels_json();
    }

    // Only allow known files
    if !matches!(
        filename.as_str(),
        "skills.yaml" | "categories.yaml" | "modules.yaml" | "tags.yaml" | 
        "fits.dat" | "fitnotes.yaml" | "skillplan.yaml"
    ) {
        return Err(Madness::BadRequest("File not editable".to_string()));
    }
    
    let file_path = format!("{}/{}", DATA_DIR, filename);
    let content = fs::read_to_string(&file_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to read file: {}", e)))?;
    
    // Return as String - Rocket.toml configures the size limit
    Ok(content)
}

#[post("/api/admin/data-files/<filename>?<kind>", data = "<input>", rank = 1)]
fn save_data_file(
    account: AuthenticatedAccount, 
    filename: String,
    kind: Option<String>,
    input: String
) -> Result<&'static str, Madness> {
    eprintln!("DEBUG: save_data_file called for filename: {}", filename);
    eprintln!("DEBUG: Input length: {} bytes", input.len());
    
    account.require_access("commanders-manage:admin")?;
    
    // Validate filename
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err(Madness::BadRequest("Invalid filename".to_string()));
    }
    
    if locales::is_locale_admin_filename(&filename) {
        if kind.as_deref() == Some("guide") {
            let (page, locale) = locales::parse_admin_filename(&filename)
                .ok_or_else(|| Madness::BadRequest("Invalid locale filename".to_string()))?;
            guides::save_locale(&page, &locale, &input)?;
            return Ok("Guide file saved successfully");
        }
        locales::save_locale_from_admin_filename(&filename, &input)?;
        return Ok("Locale file saved successfully");
    }

    if locales::is_language_labels_filename(&filename) {
        locales::save_language_labels(&input)?;
        return Ok("Language labels saved successfully");
    }

    // Only allow known files
    if !matches!(
        filename.as_str(),
        "skills.yaml" | "categories.yaml" | "modules.yaml" | "tags.yaml" | 
        "fits.dat" | "fitnotes.yaml" | "skillplan.yaml"
    ) {
        return Err(Madness::BadRequest("File not editable".to_string()));
    }
    
    eprintln!("DEBUG: Starting to process file: {}", filename);
    
    // Use the input string directly
    let content = input;
    
    // Handle file-specific save logic
    match filename.as_str() {
        "skills.yaml" => {
            eprintln!("DEBUG: Calling save_skills_to_file");
            crate::tla::skills::save_skills_to_file(&content)?;
            eprintln!("DEBUG: save_skills_to_file completed, calling reload_skill_data");
            crate::tla::skills::reload_skill_data()
                .map_err(|e| Madness::BadRequest(format!("Failed to reload skills: {}", e)))?;
            eprintln!("DEBUG: reload_skill_data completed");
        }
        "categories.yaml" => {
            crate::data::categories::save_categories_to_file(&content)?;
            crate::data::categories::reload_category_data()
                .map_err(|e| Madness::BadRequest(format!("Failed to reload categories: {}", e)))?;
        }
        "modules.yaml" => {
            crate::data::variations::save_modules_to_file(&content)?;
            crate::data::variations::reload_variations()
                .map_err(|e| Madness::BadRequest(format!("Failed to reload modules: {}", e)))?;
            crate::tla::fitmatch::reload_identifier()
                .map_err(|e| Madness::BadRequest(format!("Failed to reload identifier: {}", e)))?;
        }
        "tags.yaml" => {
            crate::data::tags::save_tags_to_file(&content)?;
            crate::data::tags::reload_tags()?;
        }
        "fits.dat" => {
            crate::data::fits::save_fits_to_file(&content)?;
            crate::data::fits::reload_fits()?;
        }
        "fitnotes.yaml" => {
            crate::routes::fittings::fitnotes::save_fitnotes_to_file(&content)?;
        }
        "skillplan.yaml" => {
            crate::data::skillplans::save_plans_from_raw_yaml(&content)?;
        }
        _ => return Err(Madness::BadRequest("Unknown file type".to_string())),
    }
    
    eprintln!("DEBUG: Successfully saved file: {}", filename);
    Ok("File saved successfully")
}

#[delete("/api/admin/data-files/<filename>")]
fn delete_data_file(account: AuthenticatedAccount, filename: String) -> Result<&'static str, Madness> {
    account.require_access("commanders-manage:admin")?;

    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err(Madness::BadRequest("Invalid filename".to_string()));
    }

    if locales::is_locale_admin_filename(&filename) {
        locales::delete_locale_from_admin_filename(&filename)?;
        return Ok("Locale file deleted successfully");
    }

    Err(Madness::BadRequest(
        "Only locale translation files can be deleted from here".to_string(),
    ))
}

#[post("/api/admin/data-files/<filename>/reload")]
fn reload_data_file(account: AuthenticatedAccount, filename: String) -> Result<&'static str, Madness> {
    account.require_access("commanders-manage:admin")?;
    
    // Validate filename
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err(Madness::BadRequest("Invalid filename".to_string()));
    }
    
    match filename.as_str() {
        "skills.yaml" => {
            crate::tla::skills::reload_skill_data()
                .map_err(|e| Madness::BadRequest(format!("Failed to reload skills: {}", e)))?;
        }
        "categories.yaml" => {
            crate::data::categories::reload_category_data()
                .map_err(|e| Madness::BadRequest(format!("Failed to reload categories: {}", e)))?;
        }
        "modules.yaml" => {
            crate::data::variations::reload_variations()
                .map_err(|e| Madness::BadRequest(format!("Failed to reload modules: {}", e)))?;
            crate::tla::fitmatch::reload_identifier()
                .map_err(|e| Madness::BadRequest(format!("Failed to reload identifier: {}", e)))?;
        }
        "tags.yaml" => {
            crate::data::tags::reload_tags()?;
        }
        "fits.dat" => {
            crate::data::fits::reload_fits()?;
        }
        "fitnotes.yaml" | "skillplan.yaml" => {
            return Err(Madness::BadRequest("File does not require reload".to_string()));
        }
        _ => return Err(Madness::BadRequest("Unknown file type".to_string())),
    }
    
    Ok("File reloaded successfully")
}

#[derive(Debug, Serialize)]
struct GuideAssetsListResponse {
    assets: Vec<guides::GuideAssetInfo>,
}

#[get("/api/admin/guides/<slug>/assets")]
fn list_guide_assets(
    account: AuthenticatedAccount,
    slug: String,
) -> Result<Json<GuideAssetsListResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;
    Ok(Json(GuideAssetsListResponse {
        assets: guides::list_assets(&slug)?,
    }))
}

#[derive(Debug, Serialize)]
struct UploadGuideAssetResponse {
    filename: String,
    size: u64,
}

#[post("/api/admin/guides/<slug>/assets/<filename>", data = "<data>")]
async fn upload_guide_asset(
    account: AuthenticatedAccount,
    slug: String,
    filename: String,
    data: Data<'_>,
) -> Result<Json<UploadGuideAssetResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;
    let bytes = data
        .open(10.megabytes())
        .into_bytes()
        .await
        .map_err(|e| Madness::BadRequest(format!("Failed to read upload: {}", e)))?;
    if !bytes.is_complete() {
        return Err(Madness::BadRequest(
            "Upload exceeds 10 MiB limit".to_string(),
        ));
    }
    let (saved_filename, size) = guides::save_asset(&slug, &filename, &bytes.into_inner())?;
    Ok(Json(UploadGuideAssetResponse {
        filename: saved_filename,
        size,
    }))
}

#[delete("/api/admin/guides/<slug>/assets/<filename>")]
fn delete_guide_asset(
    account: AuthenticatedAccount,
    slug: String,
    filename: String,
) -> Result<&'static str, Madness> {
    account.require_access("commanders-manage:admin")?;
    guides::delete_asset(&slug, &filename)?;
    Ok("Asset deleted successfully")
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        list_data_files,
        get_data_file,
        save_data_file,
        delete_data_file,
        reload_data_file,
        list_guide_assets,
        upload_guide_asset,
        delete_guide_asset,
    ]
}

