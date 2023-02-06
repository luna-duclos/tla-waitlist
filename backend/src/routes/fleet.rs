use std::collections::HashMap;

use crate::{
    app::Application,
    core::{
        auth::{authorize_character, AuthenticatedAccount},
        esi::{ESIError, ESIScope},
    },
    util::{
        self,
        madness::Madness,
        types::{Character, Hull},
    },
};
use eve_data_core::TypeDB;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct FleetStatusFleet {
    id: i64,
    boss: Character,
}

#[derive(Debug, Serialize)]
struct FleetStatusResponse {
    fleets: Vec<FleetStatusFleet>,
}

#[get("/api/fleet/status")]
async fn fleet_status(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
) -> Result<Json<FleetStatusResponse>, Madness> {
    account.require_access("fleet-view")?;

    let fleets = sqlx::query!("SELECT fleet.id, boss_id, name FROM fleet JOIN `character` ON fleet.boss_id = `character`.id").fetch_all(app.get_db()).await?.into_iter()
        .map(|fleet| FleetStatusFleet {
            id: fleet.id,
            boss: Character {
                id: fleet.boss_id,
                name: fleet.name,
                corporation_id: None,
            },
        }).collect();

    Ok(Json(FleetStatusResponse { fleets }))
}

async fn get_current_fleet_id(
    app: &rocket::State<Application>,
    character_id: i64,
) -> Result<i64, Madness> {
    #[derive(Debug, Deserialize)]
    struct BasicInfo {
        fleet_id: i64,
    }

    let basic_info = app
        .esi_client
        .get(
            &format!("/v1/characters/{}/fleet", character_id),
            character_id,
            ESIScope::Fleets_ReadFleet_v1,
        )
        .await;
    if let Err(whatswrong) = basic_info {
        match whatswrong {
            ESIError::Status(404) => return Err(Madness::NotFound("You are not in a fleet")),
            e => return Err(e.into()),
        };
    }
    let basic_info: BasicInfo = basic_info.unwrap();
    Ok(basic_info.fleet_id)
}

#[derive(Debug, Serialize)]
struct FleetInfoResponse {
    fleet_id: i64,
    wings: Vec<FleetInfoWing>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FleetInfoWing {
    id: i64,
    name: String,
    squads: Vec<FleetInfoSquad>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FleetInfoSquad {
    id: i64,
    name: String,
}

async fn fetch_fleet_wings(
    app: &rocket::State<Application>,
    character_id: i64,
    fleet_id: i64,
) -> Result<Vec<FleetInfoWing>, Madness> {
    let wings = app
        .esi_client
        .get(
            &format!("/v1/fleets/{}/wings", fleet_id),
            character_id,
            ESIScope::Fleets_ReadFleet_v1,
        )
        .await;
    if let Err(whatswrong) = wings {
        match whatswrong {
            ESIError::Status(404) => return Err(Madness::NotFound("You are not the fleet boss")),
            e => return Err(e.into()),
        };
    }
    Ok(wings?)
}

#[get("/api/fleet/info?<character_id>")]
async fn fleet_info(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
    character_id: i64,
) -> Result<Json<FleetInfoResponse>, Madness> {
    account.require_access("fleet-view")?;
    authorize_character(app.get_db(), &account, character_id, None).await?;
    let fleet_id = get_current_fleet_id(app, character_id).await?;
    let wings = fetch_fleet_wings(app, character_id, fleet_id).await?;

    Ok(Json(FleetInfoResponse { fleet_id, wings }))
}

#[derive(Debug, Serialize)]
struct FleetCompResponse {
    wings: Vec<FleetCompWing>,
    id: i64,
    member: Option<FleetMember>,
}

#[derive(Debug, Serialize)]
struct FleetCompWing {
    id: i64,
    name: String,
    squads: Vec<FleetCompSquadMembers>,
    member: Option<FleetMember>,
}

#[derive(Debug, Serialize)]
struct FleetCompSquadMembers {
    id: i64,
    name: String,
    members: Vec<FleetMember>,
}

#[derive(Debug, Serialize)]
struct FleetMember {
    id: i64,
    name: Option<String>,
    ship: Hull,
    role: String,
}

#[get("/api/fleet/fleetcomp?<character_id>")]
async fn fleet_composition(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
    character_id: i64,
) -> Result<Json<FleetCompResponse>, Madness> {
    account.require_access("fleet-view")?;
    authorize_character(app.get_db(), &account, character_id, None).await?;
    let fleet_id = get_current_fleet_id(app, character_id).await?;
    let fleet = match sqlx::query!("SELECT boss_id FROM fleet WHERE id = ?", fleet_id)
        .fetch_optional(app.get_db())
        .await?
    {
        Some(fleet) => fleet,
        None => return Err(Madness::NotFound("Fleet not configured")),
    };
    let wings_info = fetch_fleet_wings(app, character_id, fleet_id).await?;
    let members =
        crate::core::esi::fleet_members::get(&app.esi_client, fleet_id, fleet.boss_id).await?;
    let character_ids: Vec<_> = members.iter().map(|member| member.character_id).collect();
    let mut characters = crate::data::character::lookup(app.get_db(), &character_ids).await?;
    let fleet_commander = members
        .iter()
        .find(|member| member.role == "fleet_commander")
        .map(|member| FleetMember {
            id: member.character_id,
            name: characters.remove(&member.character_id).map(|f| f.name),
            ship: Hull {
                id: member.ship_type_id,
                name: TypeDB::name_of(member.ship_type_id).unwrap(),
            },
            role: member.role.clone(),
        });
    let wings = wings_info
        .into_iter()
        .map(|info_wing| FleetCompWing {
            id: info_wing.id,
            member: members
                .iter()
                .find(|member| member.wing_id == info_wing.id && member.role == "wing_commander")
                .map(|member| FleetMember {
                    id: member.character_id,
                    name: characters.remove(&member.character_id).map(|f| f.name),
                    ship: Hull {
                        id: member.ship_type_id,
                        name: TypeDB::name_of(member.ship_type_id).unwrap(),
                    },
                    role: member.role.clone(),
                }),
            name: info_wing.name,

            squads: info_wing
                .squads
                .into_iter()
                .map(|info_squad| {
                    let squad_members = members
                        .iter()
                        .filter(|member| member.squad_id == info_squad.id)
                        .map(|member| FleetMember {
                            id: member.character_id,
                            name: characters.remove(&member.character_id).map(|f| f.name),
                            ship: Hull {
                                id: member.ship_type_id,
                                name: TypeDB::name_of(member.ship_type_id).unwrap(),
                            },
                            role: member.role.clone(),
                        })
                        .collect();

                    FleetCompSquadMembers {
                        id: info_squad.id,
                        name: info_squad.name,
                        members: squad_members,
                    }
                })
                .collect(),
        })
        .collect();

    Ok(Json(FleetCompResponse {
        wings,
        id: fleet_id,
        member: fleet_commander,
    }))
}

#[derive(Debug, Serialize)]
struct FleetMembersResponse {
    members: Vec<FleetMembersMember>,
}

#[derive(Debug, Serialize)]
struct FleetMembersMember {
    id: i64,
    name: Option<String>,
    ship: Hull,
    wl_category: Option<String>,
    category: Option<String>,
    role: String,
}

#[get("/api/fleet/members?<character_id>")]
async fn fleet_members(
    account: AuthenticatedAccount,
    character_id: i64,
    app: &rocket::State<Application>,
) -> Result<Json<FleetMembersResponse>, Madness> {
    account.require_access("fleet-view")?;
    authorize_character(app.get_db(), &account, character_id, None).await?;

    let fleet_id = get_current_fleet_id(app, character_id).await?;
    let fleet = match sqlx::query!("SELECT boss_id FROM fleet WHERE id = ?", fleet_id)
        .fetch_optional(app.get_db())
        .await?
    {
        Some(fleet) => fleet,
        None => return Err(Madness::NotFound("Fleet not configured")),
    };

    let in_fleet =
        crate::core::esi::fleet_members::get(&app.esi_client, fleet_id, fleet.boss_id).await?;
    let character_ids: Vec<_> = in_fleet.iter().map(|member| member.character_id).collect();
    let mut characters = crate::data::character::lookup(app.get_db(), &character_ids).await?;

    let category_lookup: HashMap<_, _> = crate::data::categories::categories()
        .iter()
        .map(|c| (&c.id as &str, &c.name))
        .collect();

    let squads: HashMap<i64, String> = sqlx::query!(
        "SELECT squad_id, category FROM fleet_squad WHERE fleet_id = ?",
        fleet_id
    )
    .fetch_all(app.get_db())
    .await?
    .into_iter()
    .map(|squad| (squad.squad_id, squad.category))
    .collect();

    let wings = fetch_fleet_wings(app, character_id, fleet_id).await?;

    Ok(Json(FleetMembersResponse {
        members: in_fleet
            .into_iter()
            .map(|member| FleetMembersMember {
                id: member.character_id,
                name: characters.remove(&member.character_id).map(|f| f.name),
                ship: Hull {
                    id: member.ship_type_id,
                    name: TypeDB::name_of(member.ship_type_id).unwrap(),
                },
                wl_category: squads
                    .get(&member.squad_id)
                    .and_then(|s| category_lookup.get(s.as_str()))
                    .map(|s| s.to_string()),
                category: wings
                    .iter()
                    .flat_map(|wing| &wing.squads)
                    .find(|squad| squad.id == member.squad_id)
                    .map(|squad| squad.name.clone()),
                role: member.role,
            })
            .collect(),
    }))
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    character_id: i64,
    fleet_id: i64,
    assignments: HashMap<String, (i64, i64)>,
}

#[post("/api/fleet/register", data = "<input>")]
async fn register_fleet(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
    input: Json<RegisterRequest>,
) -> Result<&'static str, Madness> {
    account.require_access("fleet-configure")?;
    authorize_character(app.get_db(), &account, input.character_id, None).await?;

    let mut tx = app.get_db().begin().await?;
    sqlx::query!("DELETE FROM fleet_squad WHERE fleet_id=?", input.fleet_id)
        .execute(&mut tx)
        .await?;
    sqlx::query!(
        "REPLACE INTO fleet (id, boss_id) VALUES (?, ?)",
        input.fleet_id,
        input.character_id
    )
    .execute(&mut tx)
    .await?;

    for category in crate::data::categories::categories() {
        if let Some((wing_id, squad_id)) = input.assignments.get(&category.id) {
            sqlx::query!("INSERT INTO fleet_squad (fleet_id, wing_id, squad_id, category) VALUES (?, ?, ?, ?)",
            input.fleet_id, wing_id, squad_id, category.id).execute(&mut tx).await?;
        } else {
            return Err(Madness::BadRequest(format!(
                "Missing assignment for {}",
                category.name
            )));
        }
    }

    tx.commit().await?;

    Ok("OK")
}

#[derive(Debug, Deserialize)]
struct FleetCloseRequest {
    character_id: i64,
}

#[post("/api/fleet/close", data = "<input>")]
async fn close_fleet(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
    input: Json<FleetCloseRequest>,
) -> Result<String, Madness> {
    authorize_character(app.get_db(), &account, input.character_id, None).await?;
    account.require_access("fleet-configure")?;

    let fleet_id = get_current_fleet_id(app, input.character_id).await?;

    let in_fleet =
        crate::core::esi::fleet_members::get(&app.esi_client, fleet_id, input.character_id).await?;

    let mut success = 0;
    let total = in_fleet.len();

    for member in in_fleet {
        if member.character_id == input.character_id {
            continue;
        }

        let res = app
            .esi_client
            .delete(
                &format!("/v1/fleets/{}/members/{}/", fleet_id, member.character_id),
                input.character_id,
                ESIScope::Fleets_WriteFleet_v1,
            )
            .await;

        if let Err(e) = res {
            match e {
                ESIError::Status(code) => match code {
                    404 => continue,
                    _ => (),
                },
                _ => (),
            }

            return Err(util::madness::Madness::ESIError(e));
        }

        if res.is_ok() {
            success += 1;
        }
    }

    if (success + 1) == total {
        return Ok(format!("All fleet members kicked."));
    }

    Ok(format!(
        "Removed {} of {} fleet members.",
        success,
        (total - 1)
    ))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        fleet_status,
        fleet_info,
        close_fleet,
        fleet_members,
        register_fleet,
        fleet_composition,
    ]
}
