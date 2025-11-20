use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

use eve_data_core::{Fitting, TypeID};

type FitData = BTreeMap<TypeID, Vec<DoctrineFit>>;

lazy_static::lazy_static! {
    static ref FITS: Arc<RwLock<FitData>> = Arc::new(RwLock::new(load_fits()));
}

#[derive(Debug)]
pub struct DoctrineFit {
    pub name: String,
    pub fit: Fitting,
}

fn load_fits() -> FitData {
    let mut fits = BTreeMap::new();

    let fit_data = std::fs::read_to_string("./data/fits.dat").expect("Could not load fits.dat");
    let fit_regex = Regex::new(r#"<a href="fitting:([0-9:;_]+)">([^<]+)</a>"#).unwrap();

    for fit_match in fit_regex.captures_iter(&fit_data) {
        let dna = fit_match.get(1).unwrap().as_str();
        let fit_name = fit_match.get(2).unwrap().as_str();
        let parsed = Fitting::from_dna(dna).unwrap();
        fits.entry(parsed.hull)
            .or_insert_with(Vec::new)
            .push(DoctrineFit {
                name: fit_name.to_string(),
                fit: parsed,
            });
    }

    fits
}

pub fn get_fits() -> Arc<RwLock<FitData>> {
    FITS.clone()
}

pub fn reload_fits() -> Result<(), crate::util::madness::Madness> {
    use crate::util::madness::Madness;
    let new_fits = load_fits();
    *FITS.write().unwrap() = new_fits;
    Ok(())
}

pub fn used_module_ids() -> Vec<TypeID> {
    let fits = get_fits();
    let fits_guard = fits.read().unwrap();
    let mut ids = BTreeSet::new();
    for (&hull, fits) in fits_guard.iter() {
        ids.insert(hull);
        for fit in fits {
            for &id in fit.fit.modules.keys() {
                ids.insert(id);
            }
            for &id in fit.fit.cargo.keys() {
                ids.insert(id);
            }
        }
    }
    ids.into_iter().collect()
}

use std::fs;
use std::io::Write;
use std::path::Path;

pub fn save_fits_to_file(content: &str) -> Result<(), crate::util::madness::Madness> {
    use crate::util::madness::Madness;
    
    // Create backup
    create_backup()?;
    
    // Write to temporary file
    let temp_path = "./data/fits.dat.tmp";
    let mut temp_file = fs::File::create(temp_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create temp file: {}", e)))?;
    
    temp_file.write_all(content.as_bytes())
        .map_err(|e| Madness::BadRequest(format!("Failed to write temp file: {}", e)))?;
    
    temp_file.sync_all()
        .map_err(|e| Madness::BadRequest(format!("Failed to sync temp file: {}", e)))?;
    
    // Atomic rename
    fs::rename(temp_path, "./data/fits.dat")
        .map_err(|e| Madness::BadRequest(format!("Failed to rename temp file: {}", e)))?;
    
    Ok(())
}

pub fn create_backup() -> Result<(), crate::util::madness::Madness> {
    use crate::util::madness::Madness;
    use std::time::SystemTime;
    
    let source = "./data/fits.dat";
    if !Path::new(source).exists() {
        return Ok(());
    }
    
    let timestamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let backup_path = format!("./data/fits.dat.backup.{}", timestamp);
    
    fs::copy(source, &backup_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create backup: {}", e)))?;
    
    // Keep only last 5 backups
    let mut backups: Vec<_> = fs::read_dir("./data")
        .map_err(|e| Madness::BadRequest(format!("Failed to read data directory: {}", e)))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("fits.dat.backup.") {
                if let Some(timestamp_str) = name.strip_prefix("fits.dat.backup.") {
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
    #[test]
    fn test_load_fits() {
        let _loaded = super::get_fits();
    }
}
