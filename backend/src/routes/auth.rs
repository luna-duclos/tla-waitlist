use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::app;
use crate::core::auth::{AuthenticatedAccount, AuthenticationError, CookieSetter};
use crate::core::esi::ESIScope;
use crate::util::{madness::Madness, types};

#[derive(Serialize)]
struct WhoamiResponse {
    account_id: i64,
    access: Vec<&'static str>,
    characters: Vec<types::Character>,
}

#[get("/api/auth/whoami")]
async fn whoami(
    app: &rocket::State<app::Application>,
    account: AuthenticatedAccount,
) -> Result<Json<WhoamiResponse>, Madness> {
    let character = sqlx::query!("SELECT id, name FROM `character` WHERE id = ?", account.id)
        .fetch_one(app.get_db())
        .await?;
    let mut characters = vec![types::Character {
        id: character.id,
        name: character.name,
        corporation_id: None,
    }];

    let alts = sqlx::query!(
        "SELECT id, name FROM alt_character JOIN `character` ON alt_character.alt_id = `character`.id WHERE account_id = ?",
        account.id
    )
    .fetch_all(app.get_db())
    .await?;

    for alt in alts {
        characters.push(types::Character {
            id: alt.id,
            name: alt.name,
            corporation_id: None,
        });
    }

    let mut access_levels = Vec::new();
    for key in account.access {
        access_levels.push(key.as_str());
    }

    Ok(Json(WhoamiResponse {
        account_id: account.id,
        access: access_levels,
        characters,
    }))
}

#[get("/api/auth/logout")]
async fn logout<'r>(
    app: &rocket::State<app::Application>,
    account: Option<AuthenticatedAccount>,
) -> Result<CookieSetter, Madness> {
    if let Some(account) = account {
        sqlx::query!(
            "DELETE FROM alt_character WHERE account_id = ? OR alt_id = ?",
            account.id,
            account.id
        )
        .execute(app.get_db())
        .await?;
    }

    Ok(CookieSetter(
        "".to_string(),
        app.config.esi.url.starts_with("https:"),
    ))
}

#[get("/api/auth/login_url?<alt>&<fc>&<srp_admin>")]
fn login_url(alt: bool, fc: bool, srp_admin: bool, app: &rocket::State<app::Application>) -> String {
    let state = if srp_admin {
        "srp_admin"
    } else {
        match alt {
            true => "alt",
            false => "normal",
        }
    };

    let mut scopes = vec![
        ESIScope::PublicData,
        ESIScope::Skills_ReadSkills_v1,
        ESIScope::Clones_ReadImplants_v1,
    ];
    if fc {
        scopes.extend(vec![
            ESIScope::Fleets_ReadFleet_v1,
            ESIScope::Fleets_WriteFleet_v1,
            ESIScope::UI_OpenWindow_v1,
            ESIScope::Search_v1,
        ])
    }
    if srp_admin {
        scopes.push(ESIScope::UI_OpenWindow_v1);
    }

    format!(
        "https://login.eveonline.com/v2/oauth/authorize?response_type=code&redirect_uri={}&client_id={}&scope={}&state={}",
        app.config.esi.url,
        app.config.esi.client_id,
        scopes.iter().fold(String::new(), |acc, scope| acc + " " + scope.as_str()).trim_end(),
        state
    )
}

#[derive(Deserialize)]
struct CallbackData<'r> {
    code: &'r str,
    state: Option<&'r str>,
}

#[derive(Serialize)]
struct PublicBanPayload {
    category: String,
    expires_at: Option<i64>,
    reason: Option<String>,
}

#[post("/api/auth/cb", data = "<input>")]
async fn callback(
    input: Json<CallbackData<'_>>,
    app: &rocket::State<app::Application>,
    account_raw: Result<AuthenticatedAccount, AuthenticationError>,
) -> Result<CookieSetter, Madness> {
    let account = match account_raw {
        Err(AuthenticationError::MissingCookie) => None,
        Err(AuthenticationError::InvalidToken) => None,
        Err(AuthenticationError::DatabaseError(e)) => return Err(e.into()),
        Ok(acc) => Some(acc),
    };

    let character_id = app
        .esi_client
        .process_authorization_code(input.code)
        .await?;

    // Update the character's corporation and aliance information
    app.affiliation_service
        .update_character_affiliation(character_id)
        .await?;

    if let Some(ban) = app.ban_service.character_bans(character_id).await? {
        let ban = ban.first().unwrap();

        let payload = PublicBanPayload {
            category: ban.entity.to_owned().unwrap().category,
            expires_at: ban.revoked_at,
            reason: ban.public_reason.to_owned(),
        };

        if let Ok(json) = serde_json::to_string(&payload) {
            return Err(Madness::Forbidden(json));
        }
        return Err(Madness::BadRequest(format!(
            "You cannot login due to a ban. An error occurred when trying to retreive the details, please contact council for more information."
        )));
    }

    let (logged_in_account, window_character_id) = if let Some(state) = input.state {
        match state {
            "alt" if account.is_some() => {
                let account = account.unwrap();
                if account.id != character_id {
                    let is_admin = sqlx::query!(
                        "SELECT character_id FROM admin WHERE character_id = ?",
                        character_id
                    )
                    .fetch_optional(app.get_db())
                    .await?;

                    if is_admin.is_some() {
                        return Err(Madness::BadRequest(
                            "Character is flagged as a main and cannot be added as an alt".to_string(),
                        ));
                    }

                    sqlx::query!(
                        "REPLACE INTO alt_character (account_id, alt_id) VALUES (?, ?)",
                        account.id,
                        character_id
                    )
                    .execute(app.get_db())
                    .await?;
                }
                (account.id, None)
            }
            "srp_admin" => {
                // For SRP admin re-auth, keep the current session but store the window character ID
                if let Some(current_account) = account {
                    // Keep the current session, but store the window character ID for window opening
                    (current_account.id, Some(character_id))
                } else {
                    // If no current session, use the new character ID
                    (character_id, None)
                }
            }
            _ => (character_id, None)
        }
    } else {
        (character_id, None)
    };

    Ok(crate::core::auth::create_cookie(app, logged_in_account, window_character_id))
}

// GET version of auth callback for OAuth redirects (used by SRP setup)
#[get("/api/auth/cb?<code>&<state>")]
async fn callback_get(
    app: &rocket::State<app::Application>,
    code: String,
    state: Option<String>,
) -> Result<rocket::response::Redirect, Madness> {
    let character_id = app
        .esi_client
        .process_authorization_code(&code)
        .await?;

    // Update the character's corporation and alliance information
    app.affiliation_service
        .update_character_affiliation(character_id)
        .await?;

    // Check if this is an SRP setup
    if state.as_deref() == Some("srp_setup") {
        // Handle SRP service account setup
        let character = sqlx::query!("SELECT name FROM `character` WHERE id = ?", character_id)
            .fetch_one(app.get_db())
            .await?;

        // Get corporation info from the character
        let character_corp = sqlx::query!("SELECT corporation_id FROM `character` WHERE id = ?", character_id)
            .fetch_one(app.get_db())
            .await?;
        
        let corporation_id = character_corp.corporation_id.ok_or_else(|| {
            Madness::BadRequest("Character has no corporation ID".to_string())
        })?;
        let wallet_id = 1; // Main wallet (corporation wallets start from 1, not 1000)

        // Get the refresh token from the database
        let refresh_token_record = sqlx::query!(
            "SELECT refresh_token FROM refresh_token WHERE character_id = ?",
            character_id
        )
        .fetch_one(app.get_db())
        .await?;

        // Get the access token from the database
        let access_token_record = sqlx::query!(
            "SELECT access_token FROM access_token WHERE character_id = ?",
            character_id
        )
        .fetch_one(app.get_db())
        .await?;

        let scopes = vec!["esi-publicdata.v1", "esi-wallet.read_corporation_wallets.v1"];
        let scopes_str = scopes.join(" ");
        
        crate::data::srp::store_service_account_tokens(
            app,
            character_id,
            &character.name,
            corporation_id,
            wallet_id,
            &access_token_record.access_token,
            &refresh_token_record.refresh_token,
            chrono::Utc::now().timestamp() + 1200, // 20 minutes from now
            &scopes_str,
        ).await?;

        // Redirect to SRP page
        return Ok(rocket::response::Redirect::to("/fc/srp"));
    }

    // Regular auth flow - redirect to frontend
    Ok(rocket::response::Redirect::to("http://localhost:3000"))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![whoami, logout, login_url, callback, callback_get]
}
