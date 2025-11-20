use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::data::yamlhelper;

lazy_static::lazy_static! {
    static ref PUBLIC_TAGS: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(build_public_tags()));
}

#[derive(Debug, Deserialize, Serialize)]
struct TagFile {
    public_tags: Vec<String>,
}

fn build_public_tags() -> HashSet<String> {
    let data: TagFile = yamlhelper::from_file("./data/tags.yaml");
    data.public_tags.into_iter().collect()
}

pub fn public_tags() -> HashSet<String> {
    PUBLIC_TAGS.read().unwrap().clone()
}

pub fn reload_tags() -> Result<(), crate::util::madness::Madness> {
    let new_tags = build_public_tags();
    *PUBLIC_TAGS.write().unwrap() = new_tags;
    Ok(())
}

use std::fs;
use std::io::Write;
use std::path::Path;

pub fn save_tags_to_file(yaml_content: &str) -> Result<(), crate::util::madness::Madness> {
    use crate::util::madness::Madness;
    
    // Validate the YAML before saving
    validate_yaml(yaml_content)?;
    
    // Create backup
    create_backup()?;
    
    // Write to temporary file
    let temp_path = "./data/tags.yaml.tmp";
    let mut temp_file = fs::File::create(temp_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create temp file: {}", e)))?;
    
    temp_file.write_all(yaml_content.as_bytes())
        .map_err(|e| Madness::BadRequest(format!("Failed to write temp file: {}", e)))?;
    
    temp_file.sync_all()
        .map_err(|e| Madness::BadRequest(format!("Failed to sync temp file: {}", e)))?;
    
    // Atomic rename
    fs::rename(temp_path, "./data/tags.yaml")
        .map_err(|e| Madness::BadRequest(format!("Failed to rename temp file: {}", e)))?;
    
    Ok(())
}

pub fn validate_yaml(yaml_content: &str) -> Result<(), crate::util::madness::Madness> {
    use crate::util::madness::Madness;
    
    // Validate YAML syntax and structure
    let _: TagFile = serde_yaml::from_str(yaml_content)
        .map_err(|e| Madness::BadRequest(format!("Invalid YAML: {}", e)))?;
    Ok(())
}

pub fn create_backup() -> Result<(), crate::util::madness::Madness> {
    use crate::util::madness::Madness;
    use std::time::SystemTime;
    
    let source = "./data/tags.yaml";
    if !Path::new(source).exists() {
        return Ok(());
    }
    
    let timestamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let backup_path = format!("./data/tags.yaml.backup.{}", timestamp);
    
    fs::copy(source, &backup_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create backup: {}", e)))?;
    
    // Keep only last 5 backups
    let mut backups: Vec<_> = fs::read_dir("./data")
        .map_err(|e| Madness::BadRequest(format!("Failed to read data directory: {}", e)))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("tags.yaml.backup.") {
                if let Some(timestamp_str) = name.strip_prefix("tags.yaml.backup.") {
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
