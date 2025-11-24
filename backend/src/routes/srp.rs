use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct Empty {}
use chrono;

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
    payment_amount: Option<f64>,
    coverage_type: Option<String>,
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

#[derive(Debug, Serialize)]
struct SRPReportsResponse {
    reports: Vec<srp::SRPReport>,
}

#[derive(Debug, Serialize)]
struct SRPReportResponse {
    report: srp::SRPReport,
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
        return Ok(Json(SRPStatusResponse { 
            status: Some("Unpaid".to_string()),
            payment_amount: None,
            coverage_type: None,
        }));
    }

    let character_name = character.unwrap().name;
    let now = chrono::Utc::now().timestamp();

    // Helper function to check SRP for a character name
    async fn check_srp_for_character(app: &crate::app::Application, character_name: &str, now: i64) -> Result<Option<(i64, String, f64)>, Madness> {
        let result = sqlx::query!(
            "SELECT expires_at, coverage_type, payment_amount FROM srp_payments 
             WHERE character_name = ? AND expires_at > ? 
             ORDER BY expires_at DESC LIMIT 1",
            character_name,
            now
        )
        .fetch_optional(app.get_db())
        .await?;
        
        Ok(result.map(|r| (r.expires_at, r.coverage_type, r.payment_amount.to_string().parse::<f64>().unwrap_or(0.0))))
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

    if let Some((expires_at, coverage_type, payment_amount)) = payment {
        let status = if coverage_type == "per_focus" {
            "Paid until end of focus".to_string()
        } else {
            let expires_dt = chrono::DateTime::<chrono::Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp_opt(expires_at, 0).unwrap(),
                chrono::Utc
            );
            format!("Paid until {}", expires_dt.format("%Y-%m-%d %H:%M UTC"))
        };
        Ok(Json(SRPStatusResponse { 
            status: Some(status),
            payment_amount: Some(payment_amount),
            coverage_type: Some(coverage_type),
        }))
    } else {
        Ok(Json(SRPStatusResponse { 
            status: Some("Unpaid".to_string()),
            payment_amount: None,
            coverage_type: None,
        }))
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

// GET /api/admin/srp/reports - Get all SRP reports for admin processing
#[get("/api/admin/srp/reports")]
async fn get_srp_reports(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<SRPReportsResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let reports = srp::get_all_srp_reports(app).await?;

    Ok(Json(SRPReportsResponse { reports }))
}

// GET /api/fc/srp/reports - Get SRP reports for FCs to view
#[get("/api/fc/srp/reports")]
async fn get_fc_srp_reports(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<SRPReportsResponse>, Madness> {
    account.require_access("fleet-view")?;

    let reports = srp::get_all_srp_reports(app).await?;

    Ok(Json(SRPReportsResponse { reports }))
}

// GET /api/admin/srp/reports/{id} - Get specific SRP report details
#[get("/api/admin/srp/reports/<killmail_id>")]
async fn get_srp_report(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    killmail_id: i64,
) -> Result<Json<SRPReportResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let report = srp::get_srp_report_by_killmail_id(app, killmail_id).await?;
    
    if let Some(report) = report {
        Ok(Json(SRPReportResponse { report }))
    } else {
        Err(Madness::NotFound("SRP report not found"))
    }
}

// GET /api/admin/srp/reports/{killmail_id}/killmail - Get killmail data for an SRP report
#[get("/api/admin/srp/reports/<killmail_id>/killmail")]
async fn get_srp_report_killmail(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    killmail_id: i64,
) -> Result<Json<esi::KillmailData>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let report = srp::get_srp_report_by_killmail_id(app, killmail_id).await?;
    
    if let Some(report) = report {
        let killmail_data = srp::get_killmail_data(app, &report.killmail_link).await?;
        Ok(Json(killmail_data))
    } else {
        Err(Madness::NotFound("SRP report not found"))
    }
}

// GET /api/admin/srp/reports/{killmail_id}/killmail/enriched - Get enriched killmail data with names
#[get("/api/admin/srp/reports/<killmail_id>/killmail/enriched")]
async fn get_srp_report_killmail_enriched(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    killmail_id: i64,
) -> Result<Json<srp::EnrichedKillmailData>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let report = srp::get_srp_report_by_killmail_id(app, killmail_id).await?;
    
    if let Some(report) = report {
        let enriched_killmail_data = srp::get_enriched_killmail_data(app, &report.killmail_link).await?;
        Ok(Json(enriched_killmail_data))
    } else {
        Err(Madness::NotFound("SRP report not found"))
    }
}

#[derive(Debug, Serialize)]
struct FleetValidationResponse {
    was_in_fleet: bool,
}

#[derive(Debug, Serialize)]
struct SRPStatusAtTimeResponse {
    had_srp_coverage: bool,
    alt_needs_linking: Option<bool>,
    payment_date: Option<String>,
    payment_amount: Option<f64>,
    expires_at: Option<String>,
    coverage_character: Option<String>,
    coverage_type: Option<String>,
}

// GET /api/admin/srp/reports/{killmail_id}/fleet-validation - Check if pilot was in fleet at death time
#[get("/api/admin/srp/reports/<killmail_id>/fleet-validation")]
async fn get_srp_report_fleet_validation(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    killmail_id: i64,
) -> Result<Json<FleetValidationResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let report = srp::get_srp_report_by_killmail_id(app, killmail_id).await?;
    
    if let Some(report) = report {
        // Get killmail data with better error handling
        let killmail_data = match srp::get_killmail_data(app, &report.killmail_link).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to get killmail data: {:?}", e);
                return Err(Madness::BadRequest(format!("Failed to fetch killmail data: {}", e)));
            }
        };
        
        // Parse the killmail time to get timestamp with better error handling
        let timestamp = match chrono::DateTime::parse_from_rfc3339(&killmail_data.killmail_time) {
            Ok(time) => time.timestamp(),
            Err(e) => {
                eprintln!("Failed to parse killmail time '{}': {:?}", killmail_data.killmail_time, e);
                return Err(Madness::BadRequest(format!("Invalid killmail time format: {}", e)));
            }
        };
        
        eprintln!("Checking fleet membership for character {} at timestamp {}", 
                 killmail_data.victim.character_id.unwrap_or(0), timestamp);
        
        // Check if victim was in a fleet at death time
        let was_in_fleet = if let Some(character_id) = killmail_data.victim.character_id {
            match srp::check_fleet_membership_at_time(app, character_id, timestamp).await {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to check fleet membership: {:?}", e);
                    return Err(Madness::BadRequest(format!("Failed to check fleet membership: {}", e)));
                }
            }
        } else {
            eprintln!("No character ID found in killmail victim data");
            false
        };

        eprintln!("Fleet membership result: {}", was_in_fleet);
        Ok(Json(FleetValidationResponse { was_in_fleet }))
    } else {
        Err(Madness::NotFound("SRP report not found"))
    }
}

// GET /api/admin/srp/reports/{killmail_id}/srp-validation - Check if pilot had SRP coverage at death time
#[get("/api/admin/srp/reports/<killmail_id>/srp-validation")]
async fn get_srp_report_srp_validation(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    killmail_id: i64,
) -> Result<Json<SRPStatusAtTimeResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let report = srp::get_srp_report_by_killmail_id(app, killmail_id).await?;
    
    if let Some(report) = report {
        let killmail_data = srp::get_killmail_data(app, &report.killmail_link).await?;
        
        // Parse the killmail time to get timestamp
        let timestamp = match chrono::DateTime::parse_from_rfc3339(&killmail_data.killmail_time) {
            Ok(time) => time.timestamp(),
            Err(e) => {
                eprintln!("Failed to parse killmail time '{}': {:?}", killmail_data.killmail_time, e);
                return Err(Madness::BadRequest(format!("Invalid killmail time format: {}", e)));
            }
        };
        
        // Check if victim had SRP coverage at death time using comprehensive three-step approach
        let (had_srp_coverage, payment_date, payment_amount, expires_at, coverage_character, coverage_type) = if let Some(character_id) = killmail_data.victim.character_id {
            // Get character name from character ID
            let character_name = match sqlx::query!(
                "SELECT name FROM `character` WHERE id = ?",
                character_id
            )
            .fetch_optional(app.get_db())
            .await? {
                Some(character) => character.name,
                None => {
                    eprintln!("No character found for ID: {} - alt needs to be linked", character_id);
                    // Return special value to indicate alt needs linking
                    return Ok(Json(SRPStatusAtTimeResponse { 
                        had_srp_coverage: false,
                        alt_needs_linking: Some(true),
                        payment_date: None,
                        payment_amount: None,
                        expires_at: None,
                        coverage_character: None,
                        coverage_type: None
                    }));
                }
            };

            // Helper function to check SRP coverage for a character
            async fn check_srp_for_character(app: &crate::app::Application, character_name: &str, timestamp: i64) -> Result<Option<(String, f64, String, String)>, Madness> {
                let result = sqlx::query!(
                    "SELECT payment_amount, payment_date, expires_at FROM srp_payments 
                     WHERE character_name = ? 
                     AND payment_date <= ? 
                     AND expires_at >= ?
                     ORDER BY payment_date DESC 
                     LIMIT 1",
                    character_name,
                    timestamp,
                    timestamp
                )
                .fetch_optional(app.get_db())
                .await?;

                if let Some(payment) = result {
                    // Format dates for display
                    let payment_date = chrono::DateTime::<chrono::Utc>::from_utc(
                        chrono::NaiveDateTime::from_timestamp_opt(payment.payment_date, 0).unwrap(),
                        chrono::Utc
                    ).format("%Y-%m-%d %H:%M UTC").to_string();
                    
                    let expires_at = chrono::DateTime::<chrono::Utc>::from_utc(
                        chrono::NaiveDateTime::from_timestamp_opt(payment.expires_at, 0).unwrap(),
                        chrono::Utc
                    ).format("%Y-%m-%d %H:%M UTC").to_string();
                    
                    Ok(Some((payment_date, payment.payment_amount.to_string().parse::<f64>().unwrap_or(0.0), expires_at, character_name.to_string())))
                } else {
                    Ok(None)
                }
            }

            // Step 1: Direct character check
            if let Some((pd, pa, ea, cc)) = check_srp_for_character(app, &character_name, timestamp).await? {
                (true, Some(pd), Some(pa), Some(ea), Some(cc), Some("direct".to_string()))
            } else {
                // Step 2: Check if victim is an alt and check main character
                let main_character = sqlx::query!(
                    "SELECT c1.name as main_name FROM alt_character ac 
                     JOIN `character` c1 ON ac.account_id = c1.id 
                     JOIN `character` c2 ON ac.alt_id = c2.id 
                     WHERE c2.id = ?",
                    character_id
                )
                .fetch_optional(app.get_db())
                .await?;

                if let Some(main) = main_character {
                    // Victim is an alt, check main character
                    if let Some((pd, pa, ea, cc)) = check_srp_for_character(app, &main.main_name, timestamp).await? {
                        (true, Some(pd), Some(pa), Some(ea), Some(cc), Some("main".to_string()))
                    } else {
                        // Step 3: Check all other alts of the main character
                        let alt_characters = sqlx::query!(
                            "SELECT c2.name as alt_name FROM alt_character ac 
                             JOIN `character` c1 ON ac.account_id = c1.id 
                             JOIN `character` c2 ON ac.alt_id = c2.id 
                             WHERE c1.id = ? AND c2.id != ?",
                            sqlx::query!("SELECT account_id FROM alt_character WHERE alt_id = ?", character_id)
                                .fetch_optional(app.get_db())
                                .await?
                                .map(|r| r.account_id)
                                .unwrap_or(0),
                            character_id
                        )
                        .fetch_all(app.get_db())
                        .await?;

                        // Check each alt character
                        for alt in alt_characters {
                            if let Some((pd, pa, ea, cc)) = check_srp_for_character(app, &alt.alt_name, timestamp).await? {
                                return Ok(Json(SRPStatusAtTimeResponse { 
                                    had_srp_coverage: true,
                                    alt_needs_linking: None,
                                    payment_date: Some(pd),
                                    payment_amount: Some(pa),
                                    expires_at: Some(ea),
                                    coverage_character: Some(cc),
                                    coverage_type: Some("alt".to_string())
                                }));
                            }
                        }
                        (false, None, None, None, None, None)
                    }
                } else {
                    // Victim is main character, check all alts
                    let alt_characters = sqlx::query!(
                        "SELECT c2.name as alt_name FROM alt_character ac 
                         JOIN `character` c1 ON ac.account_id = c1.id 
                         JOIN `character` c2 ON ac.alt_id = c2.id 
                         WHERE c1.id = ?",
                        character_id
                    )
                    .fetch_all(app.get_db())
                    .await?;

                    // Check each alt character
                    for alt in alt_characters {
                        if let Some((pd, pa, ea, cc)) = check_srp_for_character(app, &alt.alt_name, timestamp).await? {
                            return Ok(Json(SRPStatusAtTimeResponse { 
                                had_srp_coverage: true,
                                alt_needs_linking: None,
                                payment_date: Some(pd),
                                payment_amount: Some(pa),
                                expires_at: Some(ea),
                                coverage_character: Some(cc),
                                coverage_type: Some("alt".to_string())
                            }));
                        }
                    }
                    (false, None, None, None, None, None)
                }
            }
        } else {
            eprintln!("No character ID found in killmail victim data");
            (false, None, None, None, None, None)
        };

        eprintln!("SRP coverage result for character ID {} at {}: {} (coverage: {:?}, type: {:?})", 
                 killmail_data.victim.character_id.unwrap_or(0), 
                 timestamp, had_srp_coverage, coverage_character, coverage_type);
        Ok(Json(SRPStatusAtTimeResponse { 
            had_srp_coverage,
            alt_needs_linking: None,
            payment_date,
            payment_amount,
            expires_at,
            coverage_character,
            coverage_type
        }))
    } else {
        Err(Madness::NotFound("SRP report not found"))
    }
}

#[derive(Debug, Deserialize)]
struct AppraisalRequest {
    destroyed_items: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AppraisalResponse {
    total_value: f64,
    item_count: usize,
    items: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct DenySRPRequest {
    reason: String,
}

#[derive(Debug, Deserialize)]
struct ApproveSRPRequest {
    payout_amount: f64,
}

#[derive(Debug, Deserialize)]
struct SubmitSRPRequest {
    killmail_link: String,
    description: String,
    loot_returned: bool,
}

#[derive(Debug, Deserialize)]
struct UpdateSRPRequest {
    description: String,
    loot_returned: bool,
}

#[derive(Debug, Serialize)]
struct SRPResponse {
    success: bool,
    message: String,
}

// POST /api/admin/srp/appraisal - Calculate appraisal for destroyed items
#[post("/api/admin/srp/appraisal", data = "<input>")]
async fn calculate_srp_appraisal(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    input: Json<AppraisalRequest>,
) -> Result<Json<AppraisalResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let (total_value, items) = srp::calculate_srp_appraisal(app, &input.destroyed_items).await?;

    Ok(Json(AppraisalResponse {
        total_value,
        item_count: input.destroyed_items.len(),
        items,
    }))
}

// POST /api/admin/srp/reports/{killmail_id}/approve - Approve an SRP report
#[post("/api/admin/srp/reports/<killmail_id>/approve", data = "<input>")]
async fn approve_srp_report(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    killmail_id: i64,
    input: Json<ApproveSRPRequest>,
) -> Result<Json<SRPResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    let result = srp::approve_srp_report(app, killmail_id, account.id, account.window_character_id, input.payout_amount).await?;

    Ok(Json(SRPResponse {
        success: true,
        message: "SRP report approved successfully".to_string(),
    }))
}

// POST /api/admin/srp/reports/{killmail_id}/deny - Deny an SRP report
#[post("/api/admin/srp/reports/<killmail_id>/deny", data = "<input>")]
async fn deny_srp_report(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    killmail_id: i64,
    input: Json<DenySRPRequest>,
) -> Result<Json<SRPResponse>, Madness> {
    eprintln!("Deny SRP report called for killmail_id: {}, reason: {}", killmail_id, input.reason);
    account.require_access("commanders-manage:admin")?;

    let result = srp::deny_srp_report(app, killmail_id, &input.reason).await?;
    eprintln!("Deny SRP report result: {:?}", result);

    Ok(Json(SRPResponse {
        success: true,
        message: "SRP report denied successfully".to_string(),
    }))
}

// POST /api/admin/srp/test-character-window - Test opening a character window
#[post("/api/admin/srp/test-character-window")]
async fn test_character_window(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<SRPResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    // Use a well-known character ID (CCP's character)
    let test_character_id = 2112625428; // CCP Falcon's character ID

    // Use window character ID if available, otherwise use account ID
    let esi_character_id = account.window_character_id.unwrap_or(account.id);

    // Open the character info window in-game
    app.esi_client
        .post(
            &format!(
                "/v1/ui/openwindow/information/?target_id={}",
                test_character_id
            ),
            &Empty {},
            esi_character_id,
            esi::ESIScope::UI_OpenWindow_v1,
        )
        .await
        .map_err(|e| {
            eprintln!("Failed to open test character window: {}", e);
            Madness::BadRequest(format!("Failed to open character window: {}", e))
        })?;

    Ok(Json(SRPResponse {
        success: true,
        message: format!("Character window opened for character ID: {} using ESI character: {}", test_character_id, esi_character_id),
    }))
}

// POST /api/admin/srp/reports/{killmail_id}/open-victim-window - Open victim's character window
#[post("/api/admin/srp/reports/<killmail_id>/open-victim-window")]
async fn open_victim_window(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    killmail_id: i64,
) -> Result<Json<SRPResponse>, Madness> {
    account.require_access("commanders-manage:admin")?;

    // Get the SRP report to get the killmail link
    let report = sqlx::query!(
        "SELECT killmail_link FROM srp_reports WHERE killmail_id = ?",
        killmail_id
    )
    .fetch_optional(app.get_db())
    .await?;

    let report = report.ok_or_else(|| Madness::NotFound("SRP report not found"))?;

    // Get the killmail data to find the victim's character ID
    let killmail_data = srp::get_killmail_data(app, &report.killmail_link).await?;
    
    // Only open character window if the victim has a character ID (not an NPC)
    if let Some(victim_character_id) = killmail_data.victim.character_id {
        // Use window character ID if available, otherwise use admin character ID
        let esi_character_id = account.window_character_id.unwrap_or(account.id);
        
        // Open the character info window in-game
        app.esi_client
            .post(
                &format!(
                    "/v1/ui/openwindow/information/?target_id={}",
                    victim_character_id
                ),
                &serde_json::json!({}),
                esi_character_id,
                esi::ESIScope::UI_OpenWindow_v1,
            )
            .await
            .map_err(|e| {
                eprintln!("Failed to open victim character window: {}", e);
                Madness::BadRequest(format!("Failed to open character window: {}", e))
            })?;

        Ok(Json(SRPResponse {
            success: true,
            message: format!("Character window opened for victim ID: {}", victim_character_id),
        }))
    } else {
        Ok(Json(SRPResponse {
            success: true,
            message: "Victim is an NPC, no character window to open".to_string(),
        }))
    }
}

#[post("/api/fc/srp/submit", data = "<input>")]
async fn submit_srp_report(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    input: Json<SubmitSRPRequest>,
) -> Result<Json<SRPResponse>, Madness> {
    account.require_access("fleet-view")?;

    srp::submit_srp_report(
        app,
        &input.killmail_link,
        &input.description,
        input.loot_returned,
        account.id,
    ).await?;

    Ok(Json(SRPResponse {
        success: true,
        message: "SRP report submitted successfully".to_string(),
    }))
}

#[post("/api/fc/srp/update/<killmail_id>", data = "<input>")]
async fn update_srp_report(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    killmail_id: i64,
    input: Json<UpdateSRPRequest>,
) -> Result<Json<SRPResponse>, Madness> {
    account.require_access("fleet-view")?;

    // Check if the user is the one who submitted this SRP report
    let report = sqlx::query!(
        "SELECT submitted_by_id FROM srp_reports WHERE killmail_id = ?",
        killmail_id
    )
    .fetch_optional(app.get_db())
    .await?;

    let report = report.ok_or_else(|| Madness::NotFound("SRP report not found"))?;
    
    if report.submitted_by_id != account.id {
        return Err(Madness::Forbidden("You can only update SRP reports that you submitted".to_string()));
    }

    srp::update_srp_report(
        app,
        killmail_id,
        &input.description,
        input.loot_returned,
    ).await?;

    Ok(Json(SRPResponse {
        success: true,
        message: "SRP report updated successfully".to_string(),
    }))
}

#[get("/api/fc/srp/report/<killmail_id>")]
async fn get_srp_report_for_edit(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    killmail_id: i64,
) -> Result<Json<serde_json::Value>, Madness> {
    account.require_access("fleet-view")?;

    // Check if the user is the one who submitted this SRP report
    let report_check = sqlx::query!(
        "SELECT submitted_by_id FROM srp_reports WHERE killmail_id = ?",
        killmail_id
    )
    .fetch_optional(app.get_db())
    .await?;

    let report_check = report_check.ok_or_else(|| Madness::NotFound("SRP report not found"))?;

    let is_owner = report_check.submitted_by_id == account.id;

    // If not owner, must have override access
    if !is_owner {
        account.require_access("commanders-manage:admin")?;
    }

    let report = srp::get_srp_report_by_killmail_id(app, killmail_id).await?;
    
    if let Some(report) = report {
        Ok(Json(serde_json::json!({
            "success": true,
            "report": {
                "killmail_id": report.killmail_id,
                "killmail_link": report.killmail_link,
                "description": report.description,
                "loot_returned": report.loot_returned
            }
        })))
    } else {
        Err(Madness::NotFound("SRP report not found"))
    }
}

#[get("/api/pilot/srp-reports/<character_id>")]
async fn get_pilot_srp_reports(
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
    character_id: i64,
) -> Result<Json<serde_json::Value>, Madness> {
    // Check if user has access to view this pilot's data
    if account.id != character_id && !account.access.contains("waitlist-manage") {
        return Err(Madness::Forbidden("Access denied".to_string()));
    }

    let reports = srp::get_srp_reports_for_pilot(app, character_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "reports": reports
    })))
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
        get_focus_end_timestamp,
        get_srp_reports,
        get_fc_srp_reports,
        get_srp_report,
        get_srp_report_killmail,
        get_srp_report_killmail_enriched,
        get_srp_report_fleet_validation,
        get_srp_report_srp_validation,
        calculate_srp_appraisal,
        approve_srp_report,
        deny_srp_report,
        test_character_window,
        open_victim_window,
        submit_srp_report,
        update_srp_report,
        get_srp_report_for_edit,
        get_pilot_srp_reports
    ]
}
