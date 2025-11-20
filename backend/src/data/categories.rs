use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::{data::yamlhelper, util::types::WaitlistCategory};
use crate::util::madness::Madness;

use eve_data_core::{Fitting, TypeDB, TypeError, TypeID};

struct CategoryData {
    categories: Vec<WaitlistCategory>,
    rules: Vec<(TypeID, String)>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CategoryRule {
    item: String,
    category: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CategoryFile {
    categories: Vec<WaitlistCategory>,
    rules: Vec<CategoryRule>,
}

lazy_static::lazy_static! {
    static ref CATEGORY_DATA: Arc<RwLock<CategoryData>> = Arc::new(RwLock::new(build_category_data().unwrap()));
}

fn build_category_data() -> Result<CategoryData, TypeError> {
    let file: CategoryFile = yamlhelper::from_file("./data/categories.yaml");

    let rules = {
        let mut rules = Vec::new();

        for rule in file.rules {
            let item = TypeDB::id_of(&rule.item)?;
            rules.push((item, rule.category));
        }

        rules
    };
    Ok(CategoryData {
        categories: file.categories,
        rules,
    })
}

pub fn categories() -> Vec<WaitlistCategory> {
    CATEGORY_DATA.read().unwrap().categories.clone()
}

pub fn rules() -> Vec<(TypeID, String)> {
    CATEGORY_DATA.read().unwrap().rules.clone()
}

pub fn reload_category_data() -> Result<(), TypeError> {
    let new_data = build_category_data()?;
    *CATEGORY_DATA.write().unwrap() = new_data;
    Ok(())
}

pub fn categorize(fit: &Fitting) -> Option<String> {
    let category_data = CATEGORY_DATA.read().unwrap();
    for (type_id, category) in &category_data.rules {
        if fit.hull == *type_id || fit.modules.contains_key(type_id) {
            return Some(category.clone());
        }
    }
    None
}

use std::fs;
use std::path::Path;

pub fn save_categories_to_file(yaml_content: &str) -> Result<(), Madness> {
    use std::io::Write;
    
    // Validate the YAML before saving
    validate_yaml(yaml_content)?;
    
    // Create backup
    create_backup()?;
    
    // Write to temporary file
    let temp_path = "./data/categories.yaml.tmp";
    let mut temp_file = fs::File::create(temp_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create temp file: {}", e)))?;
    
    temp_file.write_all(yaml_content.as_bytes())
        .map_err(|e| Madness::BadRequest(format!("Failed to write temp file: {}", e)))?;
    
    temp_file.sync_all()
        .map_err(|e| Madness::BadRequest(format!("Failed to sync temp file: {}", e)))?;
    
    // Atomic rename
    fs::rename(temp_path, "./data/categories.yaml")
        .map_err(|e| Madness::BadRequest(format!("Failed to rename temp file: {}", e)))?;
    
    Ok(())
}

pub fn validate_yaml(yaml_content: &str) -> Result<(), Madness> {
    // Validate YAML syntax and structure
    let _: CategoryFile = serde_yaml::from_str(yaml_content)
        .map_err(|e| Madness::BadRequest(format!("Invalid YAML: {}", e)))?;
    Ok(())
}

pub fn create_backup() -> Result<(), Madness> {
    use std::time::SystemTime;
    
    let source = "./data/categories.yaml";
    if !Path::new(source).exists() {
        return Ok(());
    }
    
    let timestamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let backup_path = format!("./data/categories.yaml.backup.{}", timestamp);
    
    fs::copy(source, &backup_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create backup: {}", e)))?;
    
    // Keep only last 5 backups
    let mut backups: Vec<_> = fs::read_dir("./data")
        .map_err(|e| Madness::BadRequest(format!("Failed to read data directory: {}", e)))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("categories.yaml.backup.") {
                if let Some(timestamp_str) = name.strip_prefix("categories.yaml.backup.") {
                    if let Ok(ts) = timestamp_str.parse::<u64>() {
                        return Some((entry.path(), ts));
                    }
                }
            }
            None
        })
        .collect();
    
    backups.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Remove old backups (keep last 5)
    for (path, _) in backups.into_iter().skip(5) {
        let _ = fs::remove_file(path);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::categories;

    #[test]
    fn test_data_load() {
        let cats = categories();
        assert!(!cats.is_empty());
        assert!(!cats[0].id.is_empty());
        assert!(!cats[0].name.is_empty());
    }
}
