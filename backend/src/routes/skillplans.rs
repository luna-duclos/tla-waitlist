use eve_data_core::{SkillLevel, TypeDB, TypeID};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{
    core::auth::AuthenticatedAccount,
    data::skillplans::{self, SkillPlan, SkillPlanError, SkillPlanLevel},
    util::madness::Madness,
    util::types::Hull,
};

#[derive(Debug, Serialize)]
struct SkillPlansResponse {
    plans: Vec<SkillPlansResponsePlan>,
}

#[derive(Debug, Serialize)]
struct SkillPlansResponsePlan {
    source: SkillPlan,
    levels: Vec<(TypeID, SkillLevel)>,
    ships: Vec<Hull>,
}

fn build_data() -> Result<SkillPlansResponse, SkillPlanError> {
    let plans = skillplans::load_plans_from_file();
    let mut result = Vec::new();
    for plan in plans {
        let levels = skillplans::build_plan(&plan)?;
        let mut ships = Vec::new();

        for level in &plan.plan {
            match level {
                SkillPlanLevel::Skills { from, tier: _ } => {
                    let ship_id = TypeDB::id_of(from)?;
                    if !ships.contains(&ship_id) {
                        ships.push(ship_id);
                    }
                }
                SkillPlanLevel::Fit { hull, fit: _ } => {
                    let ship_id = TypeDB::id_of(hull)?;
                    if !ships.contains(&ship_id) {
                        ships.push(ship_id);
                    }
                }
                _ => (),
            };
        }

        let ships_lookup = TypeDB::load_types(&ships)?;
        let mut hulls = Vec::new();
        for ship in ships {
            let hull = ships_lookup
                .get(&ship)
                .unwrap()
                .as_ref()
                .expect("Ships exist");
            hulls.push(Hull {
                id: ship,
                name: hull.name.clone(),
            });
        }

        result.push(SkillPlansResponsePlan {
            source: plan,
            levels,
            ships: hulls,
        });
    }

    Ok(SkillPlansResponse { plans: result })
}

fn build_data_skip_errors() -> SkillPlansResponse {
    let plans = skillplans::load_plans_from_file();
    let mut result = Vec::new();
    for plan in plans {
        // Try to build the plan, but skip it if it fails (e.g., missing fit)
        match skillplans::build_plan(&plan) {
            Ok(levels) => {
                let mut ship_names = Vec::new();
                let mut ship_ids = Vec::new();

                for level in &plan.plan {
                    match level {
                        SkillPlanLevel::Skills { from, tier: _ } => {
                            // Check if this is a tank type variant (e.g., "Armor Kronos", "Shield Kronos")
                            if from.contains("Armor") || from.contains("Shield") {
                                // This is a tank type variant - use the name directly
                                if !ship_names.contains(from) {
                                    ship_names.push(from.clone());
                                }
                            } else {
                                // Try to look it up as a ship TypeID
                                if let Ok(ship_id) = TypeDB::id_of(from) {
                                    if !ship_ids.contains(&ship_id) {
                                        ship_ids.push(ship_id);
                                    }
                                } else {
                                    // If lookup fails, might be a skill set name - use it directly
                                    if !ship_names.contains(from) {
                                        ship_names.push(from.clone());
                                    }
                                }
                            }
                        }
                        SkillPlanLevel::Fit { hull, fit } => {
                            // For Kronos, infer tank type from fit name
                            if hull == "Kronos" {
                                if fit.contains("ARMOR") {
                                    if !ship_names.contains(&"Armor Kronos".to_string()) {
                                        ship_names.push("Armor Kronos".to_string());
                                    }
                                } else if fit.contains("SHIELD") {
                                    if !ship_names.contains(&"Shield Kronos".to_string()) {
                                        ship_names.push("Shield Kronos".to_string());
                                    }
                                }
                            }
                            
                            // Also add the base hull ID for reference
                            if let Ok(ship_id) = TypeDB::id_of(hull) {
                                if !ship_ids.contains(&ship_id) {
                                    ship_ids.push(ship_id);
                                }
                            }
                        }
                        _ => (),
                    };
                }

                let mut hulls = Vec::new();
                
                // Add ship names (tank type variants) directly
                for name in ship_names {
                    // For tank type variants, we need to find the base ship ID
                    // Extract base ship name (e.g., "Kronos" from "Armor Kronos")
                    let base_ship = if name.starts_with("Armor ") {
                        name.strip_prefix("Armor ").unwrap_or(&name)
                    } else if name.starts_with("Shield ") {
                        name.strip_prefix("Shield ").unwrap_or(&name)
                    } else {
                        &name
                    };
                    
                    // Try to get the base ship ID for the hull
                    if let Ok(base_id) = TypeDB::id_of(base_ship) {
                        hulls.push(Hull {
                            id: base_id,
                            name: name.clone(),
                        });
                    } else {
                        // If we can't find the base ship, use a placeholder ID (0)
                        // The frontend will match by name
                        hulls.push(Hull {
                            id: 0,
                            name: name.clone(),
                        });
                    }
                }
                
                // Add actual ship TypeIDs
                if let Ok(ships_lookup) = TypeDB::load_types(&ship_ids) {
                    for ship_id in ship_ids {
                        if let Some(Some(hull)) = ships_lookup.get(&ship_id) {
                            // Only add if we haven't already added a tank variant for this ship
                            let ship_name = &hull.name;
                            if !hulls.iter().any(|h| h.name == *ship_name || 
                                h.name == format!("Armor {}", ship_name) || 
                                h.name == format!("Shield {}", ship_name)) {
                                hulls.push(Hull {
                                    id: ship_id,
                                    name: hull.name.clone(),
                                });
                            }
                        }
                    }
                }

                result.push(SkillPlansResponsePlan {
                    source: plan,
                    levels,
                    ships: hulls,
                });
            }
            Err(e) => {
                warn!("Skipping skill plan '{}' due to error: {}", plan.name, e);
            }
        }
    }

    SkillPlansResponse { plans: result }
}

#[get("/api/skills/plans")]
fn get_skill_plans(_account: AuthenticatedAccount) -> Result<Json<SkillPlansResponse>, Madness> {
    Ok(Json(build_data_skip_errors()))
}

// Admin routes
#[get("/api/admin/skillplans")]
fn get_admin_skill_plans(account: AuthenticatedAccount) -> Result<Json<Vec<SkillPlan>>, Madness> {
    account.require_access("commanders-manage:admin")?;
    let plans = skillplans::load_plans_from_file();
    Ok(Json(plans))
}

#[get("/api/admin/skillplans/<name>")]
fn get_admin_skill_plan(account: AuthenticatedAccount, name: String) -> Result<Json<SkillPlan>, Madness> {
    account.require_access("commanders-manage:admin")?;
    let plans = skillplans::load_plans_from_file();
    let plan = plans.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| Madness::NotFound("Skill plan not found"))?;
    Ok(Json(plan.clone()))
}

#[derive(Debug, Deserialize)]
struct CreateSkillPlanRequest {
    plan: SkillPlan,
}

#[post("/api/admin/skillplans", data = "<input>")]
fn create_admin_skill_plan(account: AuthenticatedAccount, input: Json<CreateSkillPlanRequest>) -> Result<&'static str, Madness> {
    account.require_access("commanders-manage:admin")?;
    
    // Validate the plan
    skillplans::validate_plan(&input.plan)?;
    
    // Load existing plans
    let mut plans = skillplans::load_plans_from_file();
    
    // Check if plan name already exists
    if plans.iter().any(|p| p.name == input.plan.name) {
        return Err(Madness::BadRequest(format!("Skill plan with name '{}' already exists", input.plan.name)));
    }
    
    // Add new plan
    plans.push(input.plan.clone());
    
    // Save to file
    skillplans::save_plans_to_file(&plans)?;
    
    Ok("Skill plan created successfully")
}

#[put("/api/admin/skillplans/<name>", data = "<input>")]
fn update_admin_skill_plan(account: AuthenticatedAccount, name: String, input: Json<CreateSkillPlanRequest>) -> Result<&'static str, Madness> {
    account.require_access("commanders-manage:admin")?;
    
    // Validate the plan
    skillplans::validate_plan(&input.plan)?;
    
    // Load existing plans
    let mut plans = skillplans::load_plans_from_file();
    
    // Find the plan index
    let index = plans.iter()
        .position(|p| p.name == name)
        .ok_or_else(|| Madness::NotFound("Skill plan not found"))?;
    
    // If name changed, check for conflicts
    if input.plan.name != name {
        if plans.iter().any(|p| p.name == input.plan.name) {
            return Err(Madness::BadRequest(format!("Skill plan with name '{}' already exists", input.plan.name)));
        }
    }
    
    // Update the plan
    plans[index] = input.plan.clone();
    
    // Save to file
    skillplans::save_plans_to_file(&plans)?;
    
    Ok("Skill plan updated successfully")
}

#[delete("/api/admin/skillplans/<name>")]
fn delete_admin_skill_plan(account: AuthenticatedAccount, name: String) -> Result<&'static str, Madness> {
    account.require_access("commanders-manage:admin")?;
    
    // Load existing plans
    let mut plans = skillplans::load_plans_from_file();
    
    // Find and remove the plan
    let initial_len = plans.len();
    plans.retain(|p| p.name != name);
    
    if plans.len() == initial_len {
        return Err(Madness::NotFound("Skill plan not found"));
    }
    
    // Save to file
    skillplans::save_plans_to_file(&plans)?;
    
    Ok("Skill plan deleted successfully")
}

#[get("/api/admin/skillplans/raw")]
fn get_admin_skill_plans_raw(account: AuthenticatedAccount) -> Result<String, Madness> {
    account.require_access("commanders-manage:admin")?;
    
    use std::fs;
    let yaml_content = fs::read_to_string("./data/skillplan.yaml")
        .map_err(|e| Madness::BadRequest(format!("Failed to read skillplan.yaml: {}", e)))?;
    
    Ok(yaml_content)
}

#[post("/api/admin/skillplans/raw", data = "<input>")]
fn save_admin_skill_plans_raw(account: AuthenticatedAccount, input: String) -> Result<&'static str, Madness> {
    account.require_access("commanders-manage:admin")?;
    
    use std::fs;
    use std::io::Write;
    
    // Validate the YAML before saving
    skillplans::validate_yaml(&input)?;
    
    // Create backup
    skillplans::create_backup()?;
    
    // Write to temporary file
    let temp_path = "./data/skillplan.yaml.tmp";
    let mut temp_file = fs::File::create(temp_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create temp file: {}", e)))?;
    
    temp_file.write_all(input.as_bytes())
        .map_err(|e| Madness::BadRequest(format!("Failed to write temp file: {}", e)))?;
    
    temp_file.sync_all()
        .map_err(|e| Madness::BadRequest(format!("Failed to sync temp file: {}", e)))?;
    
    // Atomic rename
    fs::rename(temp_path, "./data/skillplan.yaml")
        .map_err(|e| Madness::BadRequest(format!("Failed to rename temp file: {}", e)))?;
    
    Ok("Skill plans saved successfully")
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_skill_plans,
        get_admin_skill_plans,
        get_admin_skill_plan,
        create_admin_skill_plan,
        update_admin_skill_plan,
        delete_admin_skill_plan,
        get_admin_skill_plans_raw,
        save_admin_skill_plans_raw,
    ]
}
