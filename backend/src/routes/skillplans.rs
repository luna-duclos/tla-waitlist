use eve_data_core::{SkillLevel, TypeDB, TypeID};
use rocket::serde::json::Json;
use serde::Serialize;

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

pub fn routes() -> Vec<rocket::Route> {
    routes![get_skill_plans]
}
