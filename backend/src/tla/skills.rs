use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::data::yamlhelper;
use crate::util::madness::Madness;
use eve_data_core::{SkillLevel, TypeDB, TypeError, TypeID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum SkillTier {
    Min,
    Elite,
    Gold,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillTiers {
    min: Option<SkillLevel>,
    elite: Option<SkillLevel>,
    gold: Option<SkillLevel>,

    pub priority: i8,
}
pub type SkillRequirements = HashMap<String, HashMap<TypeID, SkillTiers>>;
pub type SkillCategories = HashMap<String, Vec<TypeID>>;

#[derive(Debug, Serialize)]
pub struct SkillData {
    pub requirements: SkillRequirements,
    pub categories: SkillCategories,
    pub relevant_skills: HashSet<TypeID>,
    pub name_lookup: HashMap<String, TypeID>,
    pub id_lookup: HashMap<TypeID, String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SkillFile {
    categories: HashMap<String, Vec<String>>,
    requirements: HashMap<String, HashMap<String, HashMap<String, SkillLevel>>>,
}

lazy_static::lazy_static! {
    static ref SKILL_DATA: Arc<RwLock<SkillData>> = Arc::new(RwLock::new(build_skill_data().unwrap()));
}

pub fn skill_data() -> Arc<RwLock<SkillData>> {
    SKILL_DATA.clone()
}

pub fn reload_skill_data() -> Result<(), TypeError> {
    eprintln!("DEBUG: reload_skill_data: Starting build_skill_data");
    let new_data = build_skill_data()?;
    eprintln!("DEBUG: reload_skill_data: build_skill_data completed, updating SKILL_DATA");
    *SKILL_DATA.write().unwrap() = new_data;
    eprintln!("DEBUG: reload_skill_data: Complete");
    Ok(())
}

fn extend_known_skills(known_skills: &mut HashSet<TypeID>) -> Result<(), TypeError> {
    // Extend known_skills with skills required to fly our fits
    {
        let mut fit_types = HashSet::new();
        let fits_data = crate::data::fits::get_fits();
        let fits_guard = fits_data.read().unwrap();
        for fit in fits_guard.values().flatten() {
            fit_types.insert(fit.fit.hull);
            for module_id in fit.fit.modules.keys() {
                fit_types.insert(*module_id);
            }
            for cargo_id in fit.fit.cargo.keys() {
                fit_types.insert(*cargo_id);
            }
        }

        let fit_types: Vec<TypeID> = fit_types.into_iter().collect();
        for the_type in TypeDB::load_types(&fit_types)?.values().flatten() {
            for &skill_id in the_type.skill_requirements.keys() {
                known_skills.insert(skill_id);
            }
        }
    }

    // Loop through known_skills and resolve requirements
    {
        let mut to_process: Vec<TypeID> = known_skills.iter().copied().collect();
        while let Some(skill_id) = to_process.pop() {
            let the_type = TypeDB::load_type(skill_id)?;
            for &requirement in the_type.skill_requirements.keys() {
                to_process.push(requirement);
                known_skills.insert(requirement);
            }
        }
    }

    Ok(())
}

fn build_skill_data() -> Result<SkillData, TypeError> {
    let skill_data: SkillFile = yamlhelper::from_file("./data/skills.yaml");

    // Build the category data. Content is {category:[..skill_ids]}
    let mut categories = HashMap::new();
    for (category_name, skill_names) in skill_data.categories {
        let mut these_skills = Vec::new();
        for skill_name in skill_names {
            these_skills.push(TypeDB::id_of(&skill_name)?);
        }
        categories.insert(category_name, these_skills);
    }

    // Build the requirement data. Content is {shipName:{skillID:{tier:level}}}, where tier is min/elite/gold
    let mut requirements = HashMap::new();
    let mut known_skills = HashSet::new();
    for (ship_name, skills) in skill_data.requirements {
        let mut these_skills = HashMap::new();
        for (skill_name, tiers) in skills {
            let min_level = tiers.get("min");
            let elite_level = tiers.get("elite").or(min_level);
            let gold_level = tiers.get("gold").or(Some(&5));

            let skill_id = TypeDB::id_of(&skill_name)?;
            these_skills.insert(
                skill_id,
                SkillTiers {
                    min: min_level.copied(),
                    elite: elite_level.copied(),
                    gold: gold_level.copied(),

                    priority: tiers.get("priority").copied().unwrap_or(1),
                },
            );
            known_skills.insert(skill_id);
        }
        requirements.insert(ship_name, these_skills);
    }

    extend_known_skills(&mut known_skills)?;

    let mut name_lookup = HashMap::new();
    let mut id_lookup = HashMap::new();
    for (id, name) in TypeDB::names_of(&known_skills.iter().copied().collect::<Vec<TypeID>>())? {
        id_lookup.insert(id, name.clone());
        name_lookup.insert(name, id);
    }

    Ok(SkillData {
        requirements,
        categories,
        relevant_skills: known_skills,
        name_lookup,
        id_lookup,
    })
}

impl SkillTiers {
    pub fn get(&self, tier: SkillTier) -> Option<SkillLevel> {
        use SkillTier::*;
        match tier {
            Min => self.min,
            Elite => self.elite,
            Gold => self.gold,
        }
    }
}

use std::path::Path;

pub fn save_skills_to_file(yaml_content: &str) -> Result<(), Madness> {
    use std::fs;
    use std::io::Write;
    
    eprintln!("DEBUG: save_skills_to_file: Starting validation");
    // Validate the YAML before saving
    validate_yaml(yaml_content)?;
    eprintln!("DEBUG: save_skills_to_file: Validation completed");
    
    eprintln!("DEBUG: save_skills_to_file: Creating backup");
    // Create backup
    create_backup()?;
    eprintln!("DEBUG: save_skills_to_file: Backup created");
    
    eprintln!("DEBUG: save_skills_to_file: Writing temp file");
    // Write to temporary file
    let temp_path = "./data/skills.yaml.tmp";
    let mut temp_file = fs::File::create(temp_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create temp file: {}", e)))?;
    
    temp_file.write_all(yaml_content.as_bytes())
        .map_err(|e| Madness::BadRequest(format!("Failed to write temp file: {}", e)))?;
    
    temp_file.sync_all()
        .map_err(|e| Madness::BadRequest(format!("Failed to sync temp file: {}", e)))?;
    eprintln!("DEBUG: save_skills_to_file: Temp file written");
    
    eprintln!("DEBUG: save_skills_to_file: Atomic rename");
    // Atomic rename
    fs::rename(temp_path, "./data/skills.yaml")
        .map_err(|e| Madness::BadRequest(format!("Failed to rename temp file: {}", e)))?;
    eprintln!("DEBUG: save_skills_to_file: Complete");
    
    Ok(())
}

pub fn validate_yaml(yaml_content: &str) -> Result<(), Madness> {
    // Parse YAML and expand merge keys (same as yamlhelper::from_file)
    let file_data: serde_yaml::Value = serde_yaml::from_str(yaml_content)
        .map_err(|e| Madness::BadRequest(format!("Invalid YAML syntax: {}", e)))?;
    
    let merged = yaml_merge_keys::merge_keys_serde(file_data)
        .map_err(|e| Madness::BadRequest(format!("Failed to merge YAML keys: {}", e)))?;
    
    let back_to_str = serde_yaml::to_string(&merged)
        .map_err(|e| Madness::BadRequest(format!("Failed to serialize merged YAML: {}", e)))?;
    
    // Validate the structure after merging
    let _: SkillFile = serde_yaml::from_str(&back_to_str)
        .map_err(|e| Madness::BadRequest(format!("Invalid YAML structure: {}", e)))?;
    Ok(())
}

pub fn create_backup() -> Result<(), Madness> {
    use std::fs;
    use std::time::SystemTime;
    
    let source = "./data/skills.yaml";
    if !Path::new(source).exists() {
        return Ok(());
    }
    
    let timestamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let backup_path = format!("./data/skills.yaml.backup.{}", timestamp);
    
    fs::copy(source, &backup_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create backup: {}", e)))?;
    
    // Keep only last 5 backups
    let mut backups: Vec<_> = fs::read_dir("./data")
        .map_err(|e| Madness::BadRequest(format!("Failed to read data directory: {}", e)))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("skills.yaml.backup.") {
                if let Some(timestamp_str) = name.strip_prefix("skills.yaml.backup.") {
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
