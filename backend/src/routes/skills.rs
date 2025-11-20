use std::collections::HashMap;

use eve_data_core::{SkillLevel, TypeID};
use rocket::serde::json::Json;
use serde::Serialize;

use crate::{
    core::auth::{authorize_character, AuthenticatedAccount},
    tla::skills as tla_skills,
    util::madness::Madness,
};

#[derive(Serialize, Debug)]
struct SkillsResponse {
    current: HashMap<TypeID, SkillLevel>,
    ids: HashMap<String, TypeID>,
    categories: tla_skills::SkillCategories,
    requirements: tla_skills::SkillRequirements,
}

#[get("/api/skills?<character_id>")]
async fn list_skills(
    app: &rocket::State<crate::app::Application>,
    character_id: i64,
    account: AuthenticatedAccount,
) -> Result<Json<SkillsResponse>, Madness> {
    authorize_character(&app.db, &account, character_id, Some("skill-view")).await?;

    let skills =
        crate::data::skills::load_skills(&app.esi_client, app.get_db(), character_id).await?;
    let skill_data = tla_skills::skill_data();
    let skill_data_guard = skill_data.read().unwrap();
    let mut relevant_skills = HashMap::new();
    for &skill_id in skill_data_guard.relevant_skills.iter() {
        relevant_skills.insert(skill_id, skills.get(skill_id));
    }

    Ok(Json(SkillsResponse {
        current: relevant_skills,
        ids: skill_data_guard.name_lookup.clone(),
        categories: skill_data_guard.categories.clone(),
        requirements: skill_data_guard.requirements.clone(),
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![list_skills]
}
