use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{
    app::Application,
    core::auth::AuthenticatedAccount,
    core::esi::{self, ESIScope},
    data::{srp, incursion},
    util::madness::Madness,
};

#[derive(Debug, Serialize)]
struct SRPStatusResponse {
    status: Option<String>,
}

#[derive(Debug, Serialize)]
struct SRPConfigResponse {
    config: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct SRPServiceAccountResponse {
    service_account: Option<srp::SRPServiceAccount>,
}

#[derive(Debug, Serialize)]
struct SRPSetupResponse {
    login_url: String,
    has_service_account: bool,
}

#[derive(Debug, Deserialize)]
struct UpdateSRPConfigRequest {
    key: String,
    value: String,
}

#[derive(Debug, Serialize)]
struct AllSRPStatusesResponse {
    statuses: Vec<srp::SRPPayment>,
}

#[derive(Debug, Serialize)]
struct SRPTestResponse {
    success: bool,
    message: String,
    entries_found: Option<i64>,
}

#[derive(Debug, Serialize)]
struct SRPWalletsResponse {
    success: bool,
    message: String,
    wallets: Option<Vec<srp::CorporationWallet>>,
}

#[derive(Debug, Serialize)]
struct SRPJournalResponse {
    success: bool,
    message: String,
    entries: Option<Vec<srp::WalletJournalEntry>>,
    wallet_id: Option<i32>,
}

#[derive(Debug, Serialize)]
struct SRPReconfigureResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct IncursionFocusResponse {
    focus_status: Option<incursion::IncursionFocus>,
}

#[derive(Debug, Serialize)]
struct FocusEndTimestampResponse {
    focus_end_timestamp: Option<i64>,
    formatted_date: Option<String>,
}

// GET /api/admin/srp/setup - Get setup status and login URL
#[get("/api/admin/srp/setup")]
async fn get_srp_setup(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<SRPSetupResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let service_account = srp::get_service_account_info(app).await?;
    let has_service_account = service_account.is_some();

    // Generate login URL with corporation wallet scope
    let scopes = vec![
        ESIScope::PublicData,
        ESIScope::Wallet_ReadCorporationWallets_v1,
    ];
    
    let scope_str = scopes.iter().fold(String::new(), |acc, scope| acc + " " + scope.as_str()).trim_end().to_string();
    
    let login_url = format!(
        "https://login.eveonline.com/v2/oauth/authorize?response_type=code&redirect_uri={}&client_id={}&scope={}&state=srp_setup",
        app.config.esi.url,
        app.config.esi.client_id,
        scope_str
    );

    Ok(Json(SRPSetupResponse {
        login_url,
        has_service_account,
    }))
}



// GET /api/admin/srp/service-account - Get current service account info
#[get("/api/admin/srp/service-account")]
async fn get_service_account(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<SRPServiceAccountResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let service_account = srp::get_service_account_info(app).await?;

    Ok(Json(SRPServiceAccountResponse { service_account }))
}

// POST /api/admin/srp/test - Test the wallet connection
#[post("/api/admin/srp/test")]
async fn test_srp_connection(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<SRPTestResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let service_account = srp::get_service_account_info(app).await?;
    if service_account.is_none() {
        return Ok(Json(SRPTestResponse {
            success: false,
            message: "No service account configured".to_string(),
            entries_found: None,
        }));
    }

    let service_account = service_account.unwrap();

    // Try to fetch wallet journal
    match srp::fetch_corporation_wallet_journal(
        app,
        service_account.character_id,
        service_account.corporation_id,
        service_account.wallet_id,
    ).await {
        Ok(entries) => {
            Ok(Json(SRPTestResponse {
                success: true,
                message: format!("Successfully connected to corporation wallet {}. Found {} entries.", service_account.wallet_id, entries.len()),
                entries_found: Some(entries.len() as i64),
            }))
        }
        Err(e) => {
            Ok(Json(SRPTestResponse {
                success: false,
                message: format!("Failed to connect to wallet {}: {} (Character: {}, Corp: {})", 
                    service_account.wallet_id, e, service_account.character_id, service_account.corporation_id),
                entries_found: None,
            }))
        }
    }
}

// POST /api/admin/srp/wallets - List available corporation wallets
#[post("/api/admin/srp/wallets")]
async fn list_corporation_wallets(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<SRPWalletsResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let service_account = srp::get_service_account_info(app).await?;
    if service_account.is_none() {
        return Ok(Json(SRPWalletsResponse {
            success: false,
            message: "No service account configured".to_string(),
            wallets: None,
        }));
    }

    let service_account = service_account.unwrap();

    // Try to fetch available wallets
    match srp::fetch_corporation_wallets(
        app,
        service_account.character_id,
        service_account.corporation_id,
    ).await {
        Ok(wallets) => {
            Ok(Json(SRPWalletsResponse {
                success: true,
                message: format!("Found {} available corporation wallets", wallets.len()),
                wallets: Some(wallets),
            }))
        }
        Err(e) => {
            Ok(Json(SRPWalletsResponse {
                success: false,
                message: format!("Failed to fetch wallets: {} (Character: {}, Corp: {})", 
                    e, service_account.character_id, service_account.corporation_id),
                wallets: None,
            }))
        }
    }
}

// POST /api/admin/srp/journal - Fetch wallet journal for first wallet
#[post("/api/admin/srp/journal")]
async fn fetch_wallet_journal(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<SRPJournalResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let service_account = srp::get_service_account_info(app).await?;
    if service_account.is_none() {
        return Ok(Json(SRPJournalResponse {
            success: false,
            message: "No service account configured".to_string(),
            entries: None,
            wallet_id: None,
        }));
    }

    let service_account = service_account.unwrap();

    // Get the first wallet division
    let wallets = srp::fetch_corporation_wallets(
        app,
        service_account.character_id,
        service_account.corporation_id,
    ).await?;

    if wallets.is_empty() {
        return Ok(Json(SRPJournalResponse {
            success: false,
            message: "No wallets found".to_string(),
            entries: None,
            wallet_id: None,
        }));
    }

    let first_wallet = &wallets[0];
    let wallet_id = first_wallet.division;

    // Fetch the wallet journal
    match srp::fetch_corporation_wallet_journal(
        app,
        service_account.character_id,
        service_account.corporation_id,
        wallet_id,
    ).await {
        Ok(entries) => {
            Ok(Json(SRPJournalResponse {
                success: true,
                message: format!("Successfully fetched journal for wallet {}. Found {} entries.", wallet_id, entries.len()),
                entries: Some(entries),
                wallet_id: Some(wallet_id),
            }))
        }
        Err(e) => {
            Ok(Json(SRPJournalResponse {
                success: false,
                message: format!("Failed to fetch journal for wallet {}: {}", wallet_id, e),
                entries: None,
                wallet_id: Some(wallet_id),
            }))
        }
    }
}

// POST /api/admin/srp/remove - Remove existing service account
#[post("/api/admin/srp/remove")]
async fn remove_service_account(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<SRPReconfigureResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    // Delete existing service account
    sqlx::query!("DELETE FROM srp_service_account")
        .execute(app.get_db())
        .await?;

    Ok(Json(SRPReconfigureResponse {
        success: true,
        message: "Service account removed. You can now set up a new service account.".to_string(),
    }))
}

// POST /api/admin/srp/process - Manually trigger SRP payment processing
#[post("/api/admin/srp/process")]
async fn process_srp_payments(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<&'static str, Madness> {
    account.require_access("commanders-manage:admin")?;

    srp::process_srp_payments(app).await?;

    Ok("SRP payments processed successfully")
}

// GET /api/srp/status?<character_id> - Get SRP status for a pilot
#[get("/api/srp/status?<character_id>")]
async fn get_srp_status(
    account: AuthenticatedAccount,
    character_id: i64,
    app: &rocket::State<Application>,
) -> Result<Json<SRPStatusResponse>, Madness> {
    // Allow pilots to check their own status, their alt's status, or FCs to check any status
    let is_own_character = character_id == account.id;
    let is_alt_of_own = sqlx::query!(
        "SELECT account_id FROM alt_character WHERE account_id = ? AND alt_id = ?",
        account.id,
        character_id
    )
    .fetch_optional(app.get_db())
    .await?
    .is_some();
    let is_own_alt = sqlx::query!(
        "SELECT account_id FROM alt_character WHERE account_id = ? AND alt_id = ?",
        character_id,
        account.id
    )
    .fetch_optional(app.get_db())
    .await?
    .is_some();
    
    if !is_own_character && !is_alt_of_own && !is_own_alt && !account.access.contains("waitlist-manage") {
        return Err(Madness::BadRequest("Unauthorized".to_string()));
    }

    // Get character name from character_id
    let character = sqlx::query!(
        "SELECT name FROM `character` WHERE id = ?",
        character_id
    )
    .fetch_optional(app.get_db())
    .await?;

    if character.is_none() {
        return Ok(Json(SRPStatusResponse { status: Some("Unpaid".to_string()) }));
    }

    let character_name = character.unwrap().name;
    let now = chrono::Utc::now().timestamp();

    // Helper function to check SRP for a character name
    async fn check_srp_for_character(app: &crate::app::Application, character_name: &str, now: i64) -> Result<Option<(i64, String)>, Madness> {
        let result = sqlx::query!(
            "SELECT expires_at, coverage_type FROM srp_payments 
             WHERE character_name = ? AND expires_at > ? 
             ORDER BY expires_at DESC LIMIT 1",
            character_name,
            now
        )
        .fetch_optional(app.get_db())
        .await?;
        
        Ok(result.map(|r| (r.expires_at, r.coverage_type)))
    }

    // First, check for active SRP payment for this character directly
    let mut payment = check_srp_for_character(app, &character_name, now).await?;

    // If no direct payment, check if this character is an alt and if the main character has SRP
    if payment.is_none() {
        if let Some(alt_relation) = sqlx::query!(
            "SELECT account_id FROM alt_character WHERE alt_id = ?",
            character_id
        )
        .fetch_optional(app.get_db())
        .await? {
            // Get the main character's name
            if let Some(main_character) = sqlx::query!(
                "SELECT name FROM `character` WHERE id = ?",
                alt_relation.account_id
            )
            .fetch_optional(app.get_db())
            .await? {
                // Check if the main character has SRP
                payment = check_srp_for_character(app, &main_character.name, now).await?;
            }
        }
    }

    // If still no payment, check if this character is a main character and if any alt has SRP
    if payment.is_none() {
        if let Some(alt_relation) = sqlx::query!(
            "SELECT alt_id FROM alt_character WHERE account_id = ?",
            character_id
        )
        .fetch_optional(app.get_db())
        .await? {
            // Get the alt character's name
            if let Some(alt_character) = sqlx::query!(
                "SELECT name FROM `character` WHERE id = ?",
                alt_relation.alt_id
            )
            .fetch_optional(app.get_db())
            .await? {
                // Check if the alt character has SRP
                payment = check_srp_for_character(app, &alt_character.name, now).await?;
            }
        }
    }

    if let Some((expires_at, coverage_type)) = payment {
        let status = if coverage_type == "per_focus" {
            "Paid until end of focus".to_string()
        } else {
            let expires_dt = chrono::DateTime::<chrono::Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp_opt(expires_at, 0).unwrap(),
                chrono::Utc
            );
            format!("Paid until {}", expires_dt.format("%Y-%m-%d %H:%M UTC"))
        };
        Ok(Json(SRPStatusResponse { status: Some(status) }))
    } else {
        Ok(Json(SRPStatusResponse { status: Some("Unpaid".to_string()) }))
    }
}

// GET /api/admin/srp/config - Get SRP configuration
#[get("/api/admin/srp/config")]
async fn get_srp_config(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<SRPConfigResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let config = srp::get_srp_config(app).await?;

    Ok(Json(SRPConfigResponse { config }))
}

// POST /api/admin/srp/config - Update SRP configuration
#[post("/api/admin/srp/config", data = "<input>")]
async fn update_srp_config(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    input: Json<UpdateSRPConfigRequest>,
) -> Result<&'static str, Madness> {
    account.require_access("commanders-manage:admin")?;

    srp::update_srp_config(app, &input.key, &input.value, account.id).await?;

    Ok("Configuration updated successfully")
}

// GET /api/admin/srp/all-statuses - Get all SRP payment statuses
#[get("/api/admin/srp/all-statuses")]
async fn get_all_srp_statuses(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<AllSRPStatusesResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let statuses = srp::get_all_srp_payments(app).await?;

    Ok(Json(AllSRPStatusesResponse { statuses }))
}

// GET /api/admin/srp/incursion-focus - Get current incursion focus status
#[get("/api/admin/srp/incursion-focus")]
async fn get_incursion_focus_status(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<IncursionFocusResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let focus_status = incursion::get_current_focus_status(app).await?;

    Ok(Json(IncursionFocusResponse { focus_status }))
}

// GET /api/admin/srp/focus-end-timestamp - Get focus end timestamp
#[get("/api/admin/srp/focus-end-timestamp")]
async fn get_focus_end_timestamp(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<FocusEndTimestampResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let result = sqlx::query!(
        "SELECT focus_end_timestamp FROM incursion_focus ORDER BY id DESC LIMIT 1"
    )
    .fetch_optional(app.get_db())
    .await?;

    let focus_end_timestamp = result.and_then(|r| r.focus_end_timestamp);
    
    let formatted_date = focus_end_timestamp.map(|timestamp| {
        let dt = chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap(),
            chrono::Utc
        );
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    });

    Ok(Json(FocusEndTimestampResponse { 
        focus_end_timestamp, 
        formatted_date 
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_srp_setup,
        get_service_account,
        test_srp_connection,
        list_corporation_wallets,
        fetch_wallet_journal,
        remove_service_account,
        process_srp_payments,
        get_srp_status,
        get_srp_config,
        update_srp_config,
        get_all_srp_statuses,
        get_incursion_focus_status,
        get_focus_end_timestamp
    ]
}
