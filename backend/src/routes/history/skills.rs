use eve_data_core::{SkillLevel, TypeID};
use rocket::serde::json::Json;
use serde::Serialize;
use std::collections::HashMap;

use crate::tla::skills as tla_skills;
use crate::{
    core::auth::{authorize_character, AuthenticatedAccount},
    util::madness::Madness,
};

#[derive(Serialize)]
struct SkillHistoryResponseLine {
    skill_id: TypeID,
    old_level: SkillLevel,
    new_level: SkillLevel,
    logged_at: i64,
    name: String,
}

#[derive(Serialize)]
struct SkillHistoryResponse {
    history: Vec<SkillHistoryResponseLine>,
    ids: HashMap<String, TypeID>,
}

#[get("/api/history/skills?<character_id>")]
async fn skill_history(
    app: &rocket::State<crate::app::Application>,
    account: AuthenticatedAccount,
    character_id: i64,
) -> Result<Json<SkillHistoryResponse>, Madness> {
    authorize_character(
        app.get_db(),
        &account,
        character_id,
        Some("skill-history-view"),
    )
    .await?;
    
    // Clone all needed data while holding the lock, then release it
    let (relevance, id_lookup, name_lookup) = {
        let skill_data = tla_skills::skill_data();
        let skill_data_guard = skill_data.read().unwrap();
        (
            skill_data_guard.relevant_skills.clone(),
            skill_data_guard.id_lookup.clone(),
            skill_data_guard.name_lookup.clone(),
        )
    };

    let history = sqlx::query!(
        "SELECT * FROM skill_history WHERE character_id = ? ORDER BY id DESC",
        character_id
    )
    .fetch_all(app.get_db())
    .await?
    .into_iter()
    .filter(|row| relevance.contains(&row.skill_id))
    .map(|row| SkillHistoryResponseLine {
        skill_id: row.skill_id as TypeID,
        old_level: row.old_level as SkillLevel,
        new_level: row.new_level as SkillLevel,
        logged_at: row.logged_at,
        name: id_lookup
            .get(&row.skill_id)
            .unwrap()
            .clone(),
    })
    .collect();

    Ok(Json(SkillHistoryResponse {
        history,
        ids: name_lookup,
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![skill_history]
}
