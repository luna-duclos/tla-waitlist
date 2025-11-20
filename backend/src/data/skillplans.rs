use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use crate::{data::yamlhelper, tla::skills::SkillTier};
use eve_data_core::{Attribute, SkillLevel, TypeDB, TypeError, TypeID};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum SkillPlanError {
    #[error("fit not found")]
    FitNotFound,

    #[error("type error")]
    TypeError(#[from] TypeError),

    #[error("invalid tier")]
    InvalidTier,

    #[error("file write error: {0}")]
    FileWriteError(String),

    #[error("validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Deserialize, Serialize)]
struct SkillPlanFile {
    plans: Vec<SkillPlan>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillPlan {
    pub name: String,
    pub description: String,
    pub plan: Vec<SkillPlanLevel>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SkillPlanLevel {
    Fit { hull: String, fit: String },
    Skills { from: String, tier: String },
    Skill { from: String, level: SkillLevel },
    Tank { from: String },
}

type LevelPair = (TypeID, SkillLevel);

pub fn build_plan(plan: &SkillPlan) -> Result<Vec<LevelPair>, SkillPlanError> {
    let mut seen = BTreeSet::new();
    let mut skills = Vec::new();

    for plan_level in &plan.plan {
        let skill_reqs = match plan_level {
            SkillPlanLevel::Fit { hull, fit } => get_fit_plan(hull, fit)?,
            SkillPlanLevel::Skills { from, tier } => get_skill_plan(from, tier)?,
            SkillPlanLevel::Skill { from, level } => get_single_skill(from, *level)?,
            SkillPlanLevel::Tank { from } => get_tank_plan(from)?,
        };

        for req in skill_reqs {
            if !seen.contains(&req) {
                seen.insert(req);
                skills.push(req);
            }
        }
    }

    Ok(skills)
}

pub fn load_plans_from_file() -> Vec<SkillPlan> {
    let file: SkillPlanFile = yamlhelper::from_file("./data/skillplan.yaml");
    file.plans
}

const SKILLPLAN_FILE: &str = "./data/skillplan.yaml";

pub fn create_backup() -> Result<String, SkillPlanError> {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to get timestamp: {}", e)))?
        .as_secs();

    let backup_path = format!("{}.backup.{}", SKILLPLAN_FILE, timestamp);
    
    fs::copy(SKILLPLAN_FILE, &backup_path)
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to create backup: {}", e)))?;

    // Clean up old backups (keep last 5)
    let mut backups: Vec<_> = fs::read_dir("./data")
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to read data directory: {}", e)))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let name = path.file_name()?.to_str()?;
            if name.starts_with("skillplan.yaml.backup.") {
                let timestamp_str = name.strip_prefix("skillplan.yaml.backup.")?;
                timestamp_str.parse::<u64>().ok().map(|ts| (ts, path))
            } else {
                None
            }
        })
        .collect();

    backups.sort_by(|a, b| b.0.cmp(&a.0)); // Sort descending (newest first)

    // Remove backups beyond the 5th one
    for (_, path) in backups.into_iter().skip(5) {
        let _ = fs::remove_file(path); // Ignore errors when cleaning up old backups
    }

    Ok(backup_path)
}

pub fn validate_yaml(yaml_content: &str) -> Result<(), SkillPlanError> {
    // Try to parse the YAML to ensure it's valid
    let file_data: serde_yaml::Value = serde_yaml::from_str(yaml_content)
        .map_err(|e| SkillPlanError::FileWriteError(format!("Invalid YAML: {}", e)))?;
    
    // Process merge keys
    let merged = yaml_merge_keys::merge_keys_serde(file_data)
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to process merge keys: {}", e)))?;
    
    // Try to deserialize as SkillPlanFile to ensure structure is correct
    let back_to_str = serde_yaml::to_string(&merged)
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to serialize YAML: {}", e)))?;
    
    let _: SkillPlanFile = serde_yaml::from_str(&back_to_str)
        .map_err(|e| SkillPlanError::FileWriteError(format!("Invalid skill plan structure: {}", e)))?;

    Ok(())
}

pub fn save_plans_to_file(plans: &[SkillPlan]) -> Result<(), SkillPlanError> {
    use std::fs;
    use std::io::Write;

    // Create backup before writing
    create_backup()?;

    // Serialize plans to YAML
    let plan_file = SkillPlanFile {
        plans: plans.to_vec(),
    };
    
    let yaml_content = serde_yaml::to_string(&plan_file)
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to serialize plans: {}", e)))?;

    // Validate the YAML before writing
    validate_yaml(&yaml_content)?;

    // Write to temporary file
    let temp_path = format!("{}.tmp", SKILLPLAN_FILE);
    let mut temp_file = fs::File::create(&temp_path)
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to create temp file: {}", e)))?;
    
    temp_file.write_all(yaml_content.as_bytes())
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to write temp file: {}", e)))?;
    
    temp_file.sync_all()
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to sync temp file: {}", e)))?;

    // Atomic rename
    fs::rename(&temp_path, SKILLPLAN_FILE)
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to rename temp file: {}", e)))?;

    Ok(())
}

#[derive(Debug, Default)]
struct DepEntry {
    dependees: Vec<LevelPair>,
    unsatisfied_requirements: usize,
    value: i64,
}

fn determine_value(
    skill: LevelPair,
    graph: &BTreeMap<LevelPair, DepEntry>,
    priority: &BTreeMap<TypeID, f64>,
    memo: &mut BTreeMap<LevelPair, f64>,
) -> Result<f64, SkillPlanError> {
    if let Some(&value) = memo.get(&skill) {
        return Ok(value);
    }

    let value = {
        let the_type = TypeDB::load_type(skill.0)?;
        let sp_needed = 250.
            * (*the_type
                .attributes
                .get(&Attribute::TrainingTimeMultiplier)
                .unwrap() as f64)
            * (f64::sqrt(32.).powi((skill.1 - 1) as i32));
        let mut value_per_sp = priority.get(&skill.0).copied().unwrap_or(1.) / sp_needed;

        let graph_entry = graph.get(&skill).unwrap();
        for &dependee in graph_entry.dependees.iter() {
            value_per_sp += determine_value(dependee, graph, priority, memo)? / 2.;
        }

        value_per_sp
    };

    memo.insert(skill, value);
    Ok(value)
}

fn create_skill_graph(
    reqs: &BTreeSet<LevelPair>,
    priority: &BTreeMap<TypeID, f64>,
) -> Result<BTreeMap<LevelPair, DepEntry>, SkillPlanError> {
    let mut requirements: BTreeMap<LevelPair, DepEntry> = BTreeMap::new();
    let mut to_process: Vec<LevelPair> = reqs.iter().copied().collect();
    let mut processed = BTreeSet::new();

    while let Some((skill_id, skill_level)) = to_process.pop() {
        if processed.contains(&(skill_id, skill_level)) {
            continue;
        }
        processed.insert((skill_id, skill_level));

        let mut these_reqs = BTreeSet::new();
        if skill_level >= 2 {
            these_reqs.insert((skill_id, skill_level - 1)); // Level N needs level N-1 trained
        } else if skill_level == 1 {
            // Only check skill requirements for level 1, or we'd be generating a very complex graph
            let the_type = TypeDB::load_type(skill_id)?;
            for (&req_id, &req_level) in the_type.skill_requirements.iter() {
                these_reqs.insert((req_id, req_level));
            }
        }

        requirements
            .entry((skill_id, skill_level))
            .or_default()
            .unsatisfied_requirements = these_reqs.len();

        for req in these_reqs {
            to_process.push(req);
            requirements
                .entry(req)
                .or_default()
                .dependees
                .push((skill_id, skill_level));
        }
    }

    let mut memo = BTreeMap::new();
    for pair in requirements.keys().copied().collect::<Vec<_>>() {
        let value = determine_value(pair, &requirements, priority, &mut memo)?;
        requirements.get_mut(&pair).unwrap().value = (value * 1000000000.) as i64;
    }

    Ok(requirements)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SkillGraphPriorityEntry {
    value: i64,
    level: SkillLevel,
    skill: TypeID,
}

fn flatten_skill_graph(mut graph: BTreeMap<LevelPair, DepEntry>) -> Vec<LevelPair> {
    let mut todo = BinaryHeap::new();
    for ((skill_id, skill_level), entry) in graph.iter() {
        if entry.unsatisfied_requirements == 0 {
            todo.push(SkillGraphPriorityEntry {
                value: entry.value,
                level: *skill_level,
                skill: *skill_id,
            });
        }
    }

    let mut order = vec![];
    while let Some(process_entry) = todo.pop() {
        let pair = (process_entry.skill, process_entry.level);
        order.push(pair);

        let entry = graph.remove(&pair).unwrap();
        for dependee in entry.dependees {
            let dependee_entry = graph.get_mut(&dependee).unwrap();
            dependee_entry.unsatisfied_requirements -= 1;
            if dependee_entry.unsatisfied_requirements == 0 {
                todo.push(SkillGraphPriorityEntry {
                    value: dependee_entry.value,
                    level: dependee.1,
                    skill: dependee.0,
                });
            }
        }
    }

    order
}

fn create_sorted_plan(
    for_hull: &str,
    requirements: &BTreeSet<LevelPair>,
) -> Result<Vec<LevelPair>, SkillPlanError> {
    let skill_data = crate::tla::skills::skill_data();
    let skill_data_guard = skill_data.read().unwrap();
    let hull_skills = skill_data_guard
        .requirements
        .get(for_hull)
        .expect("Surely we checked this by now?");

    let skill_priority = hull_skills
        .iter()
        .map(|(&skill, tiers)| (skill, tiers.priority as f64))
        .collect();

    let graph = create_skill_graph(requirements, &skill_priority)?;
    Ok(flatten_skill_graph(graph))
}

fn get_fit_plan(hull: &str, fit_name: &str) -> Result<Vec<LevelPair>, SkillPlanError> {
    let hull_id = TypeDB::id_of(hull)?;
    let fits_data = crate::data::fits::get_fits();
    let fits_guard = fits_data.read().unwrap();
    let hull_fits = match fits_guard.get(&hull_id) {
        Some(fits) => fits,
        None => return Err(SkillPlanError::FitNotFound),
    };

    if let Some(fit) = hull_fits.iter().find(|fit| fit.name == fit_name) {
        let mut to_process = vec![fit.fit.hull];
        for module_id in fit.fit.modules.keys() {
            to_process.push(*module_id);
        }

        let mut requirements = BTreeSet::new();
        for type_id in to_process {
            let the_type = TypeDB::load_type(type_id)?;
            for (&req_type, &req_level) in the_type.skill_requirements.iter() {
                requirements.insert((req_type, req_level));
            }
        }

        let hull_name = TypeDB::name_of(fit.fit.hull)?;
        
        // Map hull name to skill set name if needed
        // For ships with multiple skill sets (like Kronos), infer from fit name
        let skill_set_name = match hull_name.as_str() {
            "Kronos" => {
                if fit_name.contains("ARMOR") {
                    "Armor Kronos"
                } else if fit_name.contains("SHIELD") {
                    "Shield Kronos"
                } else {
                    "Armor Kronos" // default to Armor Kronos
                }
            },
            _ => hull_name.as_str(),
        };
        
        create_sorted_plan(skill_set_name, &requirements)
    } else {
        Err(SkillPlanError::FitNotFound)
    }
}

fn get_skill_plan(hull_name: &str, level_name: &str) -> Result<Vec<LevelPair>, SkillPlanError> {
    let tier = match level_name {
        "min" => SkillTier::Min,
        "elite" => SkillTier::Elite,
        "gold" => SkillTier::Gold,
        _ => return Err(SkillPlanError::InvalidTier),
    };

    let skill_data = crate::tla::skills::skill_data();
    let skill_data_guard = skill_data.read().unwrap();
    create_sorted_plan(
        hull_name,
        &skill_data_guard
            .requirements
            .get(hull_name)
            .expect("Expected known ship")
            .iter()
            .map(|(&skill_id, tiers)| (skill_id, tiers.get(tier).unwrap_or_default()))
            .filter(|(_skill_id, skill_level)| *skill_level > 0)
            .collect(),
    )
}

fn get_tank_plan(level_name: &str) -> Result<Vec<LevelPair>, SkillPlanError> {
    let armor_comps = match level_name {
        "starter" => 2,
        _ => 4,
    } as SkillLevel;

    let mut reqs = BTreeSet::new();
    reqs.insert((type_id!("EM Armor Compensation"), armor_comps));
    reqs.insert((type_id!("Thermal Armor Compensation"), armor_comps));
    reqs.insert((type_id!("Kinetic Armor Compensation"), armor_comps));
    reqs.insert((type_id!("Explosive Armor Compensation"), armor_comps));

    if level_name == "bastion" {
        reqs.insert((type_id!("Mechanics"), 4));
        reqs.insert((type_id!("Hull Upgrades"), 5));
    }

    // The tank skill order isn't ship-specific so just specify Megathron here
    create_sorted_plan("Megathron", &reqs)
}

fn get_single_skill(skill_name: &str, level: SkillLevel) -> Result<Vec<LevelPair>, SkillPlanError> {
    let skill_id = TypeDB::id_of(skill_name)?;
    let mut reqs = BTreeSet::new();
    reqs.insert((skill_id, level));

    // Don't know what ship it is...
    create_sorted_plan("Megathron", &reqs)
}

pub fn validate_plan(plan: &SkillPlan) -> Result<(), SkillPlanError> {
    // Validate plan name is not empty
    if plan.name.trim().is_empty() {
        return Err(SkillPlanError::ValidationError("Plan name cannot be empty".to_string()));
    }

    // Validate each plan level
    for (index, level) in plan.plan.iter().enumerate() {
        match level {
            SkillPlanLevel::Fit { hull, fit } => {
                // Validate hull exists
                TypeDB::id_of(hull)
                    .map_err(|e| SkillPlanError::ValidationError(format!(
                        "Plan step {}: Hull '{}' not found: {}", index + 1, hull, e
                    )))?;

                // Validate fit exists
                let hull_id = TypeDB::id_of(hull)?;
                let fits_data = crate::data::fits::get_fits();
                let fits_guard = fits_data.read().unwrap();
                let hull_fits = fits_guard
                    .get(&hull_id)
                    .ok_or_else(|| SkillPlanError::ValidationError(format!(
                        "Plan step {}: No fits found for hull '{}'", index + 1, hull
                    )))?;

                if !hull_fits.iter().any(|f| f.name == *fit) {
                    return Err(SkillPlanError::ValidationError(format!(
                        "Plan step {}: Fit '{}' not found for hull '{}'", index + 1, fit, hull
                    )));
                }
            }
            SkillPlanLevel::Skills { from, tier } => {
                // Validate tier is valid
                match tier.as_str() {
                    "min" | "elite" | "gold" => {}
                    _ => {
                        return Err(SkillPlanError::ValidationError(format!(
                            "Plan step {}: Invalid tier '{}'. Must be 'min', 'elite', or 'gold'", 
                            index + 1, tier
                        )));
                    }
                }

                // Validate ship/skill set name exists
                // Check if it's a tank variant (like "Armor Kronos") or regular ship name
                if from.contains("Armor ") || from.contains("Shield ") {
                    // Tank variant - check if base ship exists
                    let base_ship = if from.starts_with("Armor ") {
                        from.strip_prefix("Armor ").unwrap_or(from)
                    } else {
                        from.strip_prefix("Shield ").unwrap_or(from)
                    };
                    TypeDB::id_of(base_ship)
                        .map_err(|e| SkillPlanError::ValidationError(format!(
                            "Plan step {}: Base ship '{}' for '{}' not found: {}", 
                            index + 1, base_ship, from, e
                        )))?;
                } else {
                    // Regular ship name - check if it exists
                    TypeDB::id_of(from)
                        .map_err(|e| SkillPlanError::ValidationError(format!(
                            "Plan step {}: Ship '{}' not found: {}", index + 1, from, e
                        )))?;
                }
            }
            SkillPlanLevel::Skill { from, level } => {
                // Validate skill name exists
                TypeDB::id_of(from)
                    .map_err(|e| SkillPlanError::ValidationError(format!(
                        "Plan step {}: Skill '{}' not found: {}", index + 1, from, e
                    )))?;

                // Validate level is 1-5
                if *level < 1 || *level > 5 {
                    return Err(SkillPlanError::ValidationError(format!(
                        "Plan step {}: Skill level must be between 1 and 5, got {}", 
                        index + 1, level
                    )));
                }
            }
            SkillPlanLevel::Tank { from } => {
                // Validate tank level name
                match from.as_str() {
                    "starter" | "min" | "elite" | "gold" | "bastion" => {}
                    _ => {
                        return Err(SkillPlanError::ValidationError(format!(
                            "Plan step {}: Invalid tank level '{}'. Must be 'starter', 'min', 'elite', 'gold', or 'bastion'", 
                            index + 1, from
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn save_plans_from_raw_yaml(yaml_content: &str) -> Result<(), SkillPlanError> {
    // Validate the YAML before saving
    validate_yaml(yaml_content)?;
    
    // Create backup
    create_backup()?;
    
    use std::fs;
    use std::io::Write;
    
    // Write to temporary file
    let temp_path = format!("{}.tmp", SKILLPLAN_FILE);
    let mut temp_file = fs::File::create(&temp_path)
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to create temp file: {}", e)))?;
    
    temp_file.write_all(yaml_content.as_bytes())
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to write temp file: {}", e)))?;
    
    temp_file.sync_all()
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to sync temp file: {}", e)))?;
    
    // Atomic rename
    fs::rename(&temp_path, SKILLPLAN_FILE)
        .map_err(|e| SkillPlanError::FileWriteError(format!("Failed to rename temp file: {}", e)))?;
    
    Ok(())
}
