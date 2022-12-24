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
    ids: &'static HashMap<String, TypeID>,
    categories: &'static tla_skills::SkillCategories,
    requirements: &'static tla_skills::SkillRequirements,
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
    let mut relevant_skills = HashMap::new();
    for &skill_id in tla_skills::skill_data().relevant_skills.iter() {
        relevant_skills.insert(skill_id, skills.get(skill_id));
    }

    Ok(Json(SkillsResponse {
        current: relevant_skills,
        ids: &tla_skills::skill_data().name_lookup,
        categories: &tla_skills::skill_data().categories,
        requirements: &tla_skills::skill_data().requirements,
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![list_skills]
}
