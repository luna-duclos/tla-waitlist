use crate::core::esi::{self, ESIScope, KillmailData};
use crate::util::madness::Madness;
use serde::{Deserialize, Serialize};
use sqlx::Row;

// SRP payment amount mapping
const DAILY_PAYMENTS: &[(f64, &str)] = &[
    (20.0, "daily"),   // 20M
    (35.0, "daily"),   // 35M
    (45.0, "daily"),   // 45M
    (50.0, "daily"),   // 50M
    (55.0, "daily"),   // 55M
    (60.0, "daily"),   // 60M
    (65.0, "daily"),   // 65M
    (70.0, "daily"),   // 70M
    (75.0, "daily"),   // 75M
    (80.0, "daily"),   // 80M
];

const PER_FOCUS_PAYMENTS: &[(f64, &str)] = &[
    (125.0, "per_focus"),
    (225.0, "per_focus"),
    (295.0, "per_focus"),
    (330.0, "per_focus"),
    (365.0, "per_focus"),
    (400.0, "per_focus"),
    (435.0, "per_focus"),
    (470.0, "per_focus"),
    (505.0, "per_focus"),
    (540.0, "per_focus"),
    (600.0, "per_focus"),
    (680.0, "per_focus"),
];

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WalletJournalEntry {
    pub amount: f64,
    pub balance: f64,
    pub context_id: Option<i64>,
    pub context_id_type: Option<String>,
    pub date: String,
    pub description: String,
    pub first_party_id: Option<i64>,
    pub first_party_type: Option<String>,
    pub id: i64,
    pub reason: Option<String>,
    pub ref_type: String,
    pub second_party_id: Option<i64>,
    pub second_party_type: Option<String>,
    pub tax: Option<f64>,
    pub tax_receiver_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SRPPayment {
    pub id: i64,
    pub character_name: String,
    pub payment_amount: f64,
    pub payment_date: i64,
    pub expires_at: i64,
    pub coverage_type: String,
    pub created_at: i64,
    pub focus_voided_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SRPServiceAccount {
    pub character_id: i64,
    pub character_name: String,
    pub corporation_id: i64,
    pub wallet_id: i32,
    pub is_active: bool,
    pub last_used: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CorporationWallet {
    pub division: i32,
    pub balance: f64,
}

#[derive(Debug, Serialize)]
pub struct SRPReport {
    pub killmail_id: i64,
    pub killmail_link: String,
    pub submitted_at: i64,
    pub loot_returned: bool,
    pub description: Option<String>,
    pub submitted_by_id: i64,
    pub submitted_by_name: String,
    pub status: String,
    pub payout_amount: Option<f64>,
    pub payout_date: Option<i64>,
    pub srp_paid: Option<String>, // JSON string containing SRP validation details
    pub reason: Option<String>,
    pub victim_character_name: Option<String>,
    pub victim_ship_type: Option<String>,
}

pub async fn fetch_corporation_wallets(
    app: &crate::app::Application,
    character_id: i64,
    corporation_id: i64,
) -> Result<Vec<CorporationWallet>, Madness> {
    let wallets: Vec<CorporationWallet> = app
        .esi_client
        .get(
            &format!("/v1/corporations/{}/wallets/", corporation_id),
            character_id,
            ESIScope::Wallet_ReadCorporationWallets_v1,
        )
        .await?;

    Ok(wallets)
}

pub async fn fetch_corporation_wallet_journal(
    app: &crate::app::Application,
    character_id: i64,
    corporation_id: i64,
    wallet_id: i32,
) -> Result<Vec<WalletJournalEntry>, Madness> {
            // println!("Fetching wallet journal for corp {} wallet {} using character {}", corporation_id, wallet_id, character_id);
    
    // ETAG DISABLED: Get the last ETag from the database
    // let last_etag = sqlx::query!(
    //     "SELECT etag FROM srp_config WHERE `key` = 'wallet_journal_etag'"
    // )
    // .fetch_optional(app.get_db())
    // .await?
    // .and_then(|r| r.etag);

    // println!("Last ETag: {:?}", last_etag);
            // println!("DEBUG: ETag disabled - requesting fresh data");

    // Make the ESI request without ETag to get fresh data
    let response = match app.esi_client
        .get_with_etag::<Vec<WalletJournalEntry>>(
            &format!("/v6/corporations/{}/wallets/{}/journal/", corporation_id, wallet_id),
            character_id,
            ESIScope::Wallet_ReadCorporationWallets_v1,
            None, // ETag disabled
        )
        .await {
            Ok(response) => {
                // println!("ESI request successful, got {} entries", response.data.len());
                response
            },
            Err(e) => {
                // println!("ESI request failed: {:?}", e);
                return Err(Madness::BadRequest(format!("ESI request failed: {:?}", e)));
            }
        };

    // ETAG DISABLED: Store the new ETag if we got one
    // if let Some(new_etag) = &response.etag {
    //     println!("DEBUG: Storing new ETag: {}", new_etag);
    //     sqlx::query!(
    //         "INSERT INTO srp_config (`key`, value, etag, updated_at, updated_by_id)
    //          VALUES (?, ?, ?, ?, ?)
    //          ON DUPLICATE KEY UPDATE
    //          etag = VALUES(etag),
    //          updated_at = VALUES(updated_at),
    //          updated_by_id = VALUES(updated_by_id)",
    //         "wallet_journal_etag",
    //         "", // Empty value since we're only storing ETag
    //         new_etag,
    //         chrono::Utc::now().timestamp(),
    //         character_id
    //     )
    //     .execute(app.get_db())
    //     .await?;
    // } else {
    //     println!("DEBUG: No new ETag received (304 Not Modified)");
    // }
    
            // println!("DEBUG: ETag storage disabled - not storing response ETag");

    Ok(response.data)
}



// Helper function to determine payment type and calculate coverage end date
fn determine_srp_coverage(payment_amount: f64, payment_date: chrono::DateTime<chrono::Utc>) -> Option<(String, chrono::DateTime<chrono::Utc>)> {
    // Convert payment amount from millions to decimal format (e.g., 20000000 -> 20.0)
    let payment_amount_decimal = payment_amount / 1_000_000.0;
            // println!("Checking payment amount: {} ({} millions) against SRP tables", payment_amount, payment_amount_decimal);
    
    // Check if it's a daily payment
    for (amount, _) in DAILY_PAYMENTS {
        if (payment_amount_decimal - amount).abs() < 0.01 {
            // Daily payment: coverage until the next 11:00 UTC
            let payment_date_naive = payment_date.naive_utc();
            let today_11am_time = chrono::NaiveTime::from_hms_opt(11, 0, 0).unwrap();
            
            // println!("DEBUG: Payment date: {}, time: {}, 11am time: {}", 
            //          payment_date_naive.date(), payment_date_naive.time(), today_11am_time);
            
            let coverage_end = if payment_date_naive.time() < today_11am_time {
                // Payment made before 11:00 UTC today, expires at 11:00 UTC today
                let today_11am = payment_date_naive.date().and_hms_opt(11, 0, 0).unwrap();
                let result = chrono::DateTime::<chrono::Utc>::from_utc(today_11am, chrono::Utc);
                // println!("Payment at {} (before 11:00) -> expires at {}", payment_date_naive, result);
                result
            } else {
                // Payment made after 11:00 UTC today, expires at 11:00 UTC tomorrow
                let tomorrow_11am = (payment_date_naive.date() + chrono::Duration::days(1)).and_hms_opt(11, 0, 0).unwrap();
                let result = chrono::DateTime::<chrono::Utc>::from_utc(tomorrow_11am, chrono::Utc);
                // println!("Payment at {} (after 11:00) -> expires at {}", payment_date_naive, result);
                result
            };
            return Some(("daily".to_string(), coverage_end));
        }
    }

    // Check if it's a per focus payment
    for (amount, _) in PER_FOCUS_PAYMENTS {
        if (payment_amount_decimal - amount).abs() < 0.01 {
            // Per focus payment: coverage for 8 days from payment date
            let coverage_end = payment_date + chrono::Duration::days(8);
            return Some(("per_focus".to_string(), coverage_end));
        }
    }

    None
}

// Helper function to find approved SRP reports that match a payment
async fn find_matching_srp_report(
    _app: &crate::app::Application,
    character_name: &str,
    payout_amount: f64,
    payment_date: chrono::DateTime<chrono::Utc>,
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
) -> Result<Option<i64>, Madness> {
    // Look for approved SRP reports for this character with matching payout amount
    // Allow for a small time window (payment could be made up to 7 days after approval)
    let payment_timestamp = payment_date.timestamp();
    let seven_days_before = payment_timestamp - (7 * 24 * 60 * 60);
    
    let result = sqlx::query!(
        "SELECT killmail_id FROM srp_reports 
         WHERE status = 'approved' 
         AND payout_amount = ? 
         AND submitted_at >= ? 
         AND submitted_at <= ?
         AND victim_character_name = ?",
        payout_amount,
        seven_days_before,
        payment_timestamp,
        character_name
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(result.map(|r| r.killmail_id))
}

// Helper function to process SRP payouts (outgoing payments)
async fn process_srp_payout(
    app: &crate::app::Application,
    entry: &WalletJournalEntry,
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
) -> Result<(), Madness> {
    // Parse the entry date
    let entry_date: chrono::DateTime<chrono::Utc> = chrono::DateTime::parse_from_rfc3339(&entry.date)
        .map_err(|_| Madness::BadRequest("Invalid date format".to_string()))?
        .into();

    // Extract character name from description
    // Format: "Character Name withdrew cash from [any] account"
    let character_name = if entry.description.contains(" withdrew cash from ") {
        let parts: Vec<&str> = entry.description.split(" withdrew cash from ").collect();
        if parts.len() > 0 {
            parts[0].to_string()
        } else {
            "Unknown".to_string()
        }
    } else if entry.description.contains(" transferred cash from ") {
        // Handle corporate transfers - extract the recipient (after "to")
        if entry.description.contains(" to ") {
            let parts: Vec<&str> = entry.description.split(" to ").collect();
            if parts.len() > 1 {
                // Extract name before "'s account"
                let recipient_part = parts[1];
                if recipient_part.contains("'s account") {
                    let name_parts: Vec<&str> = recipient_part.split("'s account").collect();
                    name_parts[0].to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        }
    } else if entry.description.contains(" transferred cash to ") {
        // Handle corporate transfers - extract the recipient (after "to")
        let parts: Vec<&str> = entry.description.split(" transferred cash to ").collect();
        if parts.len() > 1 {
            // Extract name before "'s account"
            let recipient_part = parts[1];
            if recipient_part.contains("'s account") {
                let name_parts: Vec<&str> = recipient_part.split("'s account").collect();
                name_parts[0].to_string()
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        }
    } else {
        "Unknown".to_string()
    };

    // Only process if we got a valid character name
    if character_name == "Unknown" {
        return Ok(());
    }

    // Convert negative amount to positive for matching
    let payout_amount = entry.amount.abs();

    // Find matching approved SRP report
    if let Some(report_id) = find_matching_srp_report(app, &character_name, payout_amount, entry_date, tx).await? {
        // Update the SRP report to paid status
        sqlx::query!(
            "UPDATE srp_reports 
             SET status = 'paid', 
                 payout_date = ? 
             WHERE killmail_id = ?",
            entry_date.timestamp(),
            report_id
        )
        .execute(&mut **tx)
        .await?;

        println!("Updated SRP report {} to paid status for {} (amount: {})", 
                 report_id, character_name, payout_amount);
    }

    Ok(())
}

pub async fn process_srp_payments(app: &crate::app::Application) -> Result<(), Madness> {
    println!("Starting SRP payment processing...");
    
    let service_account = get_service_account_info(app).await?;
    if service_account.is_none() {
        // println!("No SRP service account configured");
        return Err(Madness::BadRequest("No SRP service account configured".to_string()));
    }
    
    let service_account = service_account.unwrap();
    // println!("Using service account: {} (ID: {}) for corp {} wallet {}", 
    //          service_account.character_name, service_account.character_id, 
    //          service_account.corporation_id, service_account.wallet_id);
    
    // Calculate date range: 8 days ago for initial scan
    let now = chrono::Utc::now();
    let eight_days_ago = now - chrono::Duration::days(8);
    // println!("Scanning for payments from {} to {}", eight_days_ago, now);

    // Fetch wallet journal entries
    let entries = fetch_corporation_wallet_journal(
        app,
        service_account.character_id,
        service_account.corporation_id,
        service_account.wallet_id
    ).await?;

    // If no entries returned (ETag disabled, so this means no data available)
    if entries.is_empty() {
        // println!("No wallet journal entries found (ETag disabled - fresh request returned no data)");
        // println!("DEBUG: This could mean either:");
        // println!("  1. Wallet journal is completely empty");
        // println!("  2. ESI endpoint issue or permissions problem");
        // println!("  3. No transactions in the specified time range");
        return Ok(());
    }

    // println!("DEBUG: Got {} new wallet journal entries", entries.len());

    // Only clear the database if we have new data to process
    // println!("New data found, clearing existing SRP payments from database");
    sqlx::query!("DELETE FROM srp_payments")
        .execute(app.get_db())
        .await?;
    // println!("Cleared existing SRP payments from database");

    // Process entries for SRP payments
    let mut tx = app.get_db().begin().await?;
    let now_timestamp = now.timestamp();

    for entry in entries {
        // Parse the entry date
        let entry_date: chrono::DateTime<chrono::Utc> = chrono::DateTime::parse_from_rfc3339(&entry.date)
            .map_err(|_| Madness::BadRequest("Invalid date format".to_string()))?
            .into(); // Convert to UTC

        // Only process entries from the last 8 days
        if entry_date < eight_days_ago {
            continue;
        }

        if entry.amount > 0.0 {
            // Process incoming SRP payments (positive amounts)
            
            // Check if this entry has already been processed
            let exists = sqlx::query!(
                "SELECT id FROM srp_payments WHERE payment_date = ? AND payment_amount = ?",
                entry_date.timestamp(),
                entry.amount
            )
            .fetch_optional(&mut tx)
            .await?;

            if exists.is_some() {
                continue; // Already processed
            }

            // Debug: Print all incoming payments
            // println!("Processing entry: amount={} ({} millions), date={}, description={}", 
            //          entry.amount, entry.amount / 1_000_000.0, entry.date, entry.description);
            
            // Determine if this is an SRP payment and calculate coverage
            if let Some((coverage_type, coverage_end)) = determine_srp_coverage(entry.amount, entry_date) {
                // println!("Found SRP payment: amount={}, type={}, coverage_end={}", entry.amount, coverage_type, coverage_end);
                
                // Extract character name from description
                // Format: "Character Name deposited cash into [any] account"
                let character_name = if entry.description.contains(" deposited cash into ") {
                    let parts: Vec<&str> = entry.description.split(" deposited cash into ").collect();
                    if parts.len() > 0 {
                        parts[0].to_string()
                    } else {
                        "Unknown".to_string()
                    }
                } else if entry.description.contains(" transferred cash from ") {
                    // Handle corporate transfers - extract the sender (first character name)
                    let parts: Vec<&str> = entry.description.split(" transferred cash from ").collect();
                    if parts.len() > 0 {
                        parts[0].to_string()
                    } else {
                        "Unknown".to_string()
                    }
                } else if entry.description.contains(" transferred cash to ") {
                    // Handle corporate transfers - extract the sender (first character name)
                    let parts: Vec<&str> = entry.description.split(" transferred cash to ").collect();
                    if parts.len() > 0 {
                        parts[0].to_string()
                    } else {
                        "Unknown".to_string()
                    }
                } else {
                    "Unknown".to_string()
                };

                // Only process if we got a valid character name
                if character_name != "Unknown" {
                    // println!("Creating SRP payment for: {} (amount: {}, expires: {})", character_name, entry.amount, coverage_end);
                    
                    // Insert SRP payment
                    sqlx::query!(
                        "INSERT INTO srp_payments (
                            character_name, payment_amount, payment_date, expires_at, coverage_type, created_at
                        ) VALUES (?, ?, ?, ?, ?, ?)",
                        character_name,
                        entry.amount,
                        entry_date.timestamp(),
                        coverage_end.timestamp(),
                        coverage_type,
                        now_timestamp
                    )
                    .execute(&mut tx)
                    .await?;
                    
                    // println!("Successfully created SRP payment for {}", character_name);
                }
            }
        } else if entry.amount < 0.0 {
            // Process outgoing SRP payouts (negative amounts)
            process_srp_payout(app, &entry, &mut tx).await?;
        }
    }

    // Update last_used timestamp
    sqlx::query!(
        "UPDATE srp_service_account SET last_used = ? WHERE character_id = ?",
        now_timestamp,
        service_account.character_id
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_all_srp_payments(
    app: &crate::app::Application,
) -> Result<Vec<SRPPayment>, Madness> {
    let now = chrono::Utc::now().timestamp();
    let now_dt = chrono::Utc::now();
    // println!("Fetching SRP payments, current time: {} ({})", now, now_dt);
    
    let results = sqlx::query!(
        "SELECT id, character_name, payment_amount, payment_date, expires_at, coverage_type, created_at, focus_voided_at
         FROM srp_payments 
         ORDER BY expires_at DESC"
    )
    .fetch_all(app.get_db())
    .await?;
    
    // println!("Found {} SRP payments in database", results.len());
    
    for result in &results {
        let expires_dt = chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp_opt(result.expires_at, 0).unwrap(),
            chrono::Utc
        );
        let is_expired = result.expires_at <= now;
        //println!("Payment: {} - {} ISK - expires at {} ({}) - expired: {} (coverage: {})", 
        //         result.character_name, result.payment_amount, result.expires_at, expires_dt, is_expired, result.coverage_type);
    }

    Ok(results
        .into_iter()
        .map(|r| SRPPayment {
            id: r.id,
            character_name: r.character_name,
            payment_amount: r.payment_amount.to_string().parse::<f64>().unwrap_or(0.0),
            payment_date: r.payment_date,
            expires_at: r.expires_at,
            coverage_type: r.coverage_type,
            created_at: r.created_at,
            focus_voided_at: r.focus_voided_at,
        })
        .collect())
}

pub async fn get_srp_config(app: &crate::app::Application) -> Result<std::collections::HashMap<String, String>, Madness> {
    let configs = sqlx::query!(
        "SELECT `key`, value FROM srp_config"
    )
    .fetch_all(app.get_db())
    .await?;

    let mut config_map = std::collections::HashMap::new();
    for config in configs {
        config_map.insert(config.key, config.value);
    }

    Ok(config_map)
}

pub async fn update_srp_config(
    app: &crate::app::Application,
    key: &str,
    value: &str,
    updated_by_id: i64,
) -> Result<(), Madness> {
    let now = chrono::Utc::now().timestamp();

    sqlx::query!(
        "INSERT INTO srp_config (`key`, value, updated_at, updated_by_id) 
         VALUES (?, ?, ?, ?) 
         ON DUPLICATE KEY UPDATE 
         value = VALUES(value), 
         updated_at = VALUES(updated_at), 
         updated_by_id = VALUES(updated_by_id)",
        key,
        value,
        now,
        updated_by_id
    )
    .execute(app.get_db())
    .await?;

    Ok(())
}

pub async fn store_service_account_tokens(
    app: &crate::app::Application,
    character_id: i64,
    character_name: &str,
    corporation_id: i64,
    wallet_id: i32,
    access_token: &str,
    refresh_token: &str,
    expires: i64,
    scopes: &str,
) -> Result<(), Madness> {
    let now = chrono::Utc::now().timestamp();

    sqlx::query!(
        "INSERT INTO srp_service_account (
            character_id, character_name, corporation_id, wallet_id,
            access_token, refresh_token, expires, scopes, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            character_name = VALUES(character_name),
            corporation_id = VALUES(corporation_id),
            wallet_id = VALUES(wallet_id),
            access_token = VALUES(access_token),
            refresh_token = VALUES(refresh_token),
            expires = VALUES(expires),
            scopes = VALUES(scopes),
            updated_at = VALUES(updated_at)",
        character_id,
        character_name,
        corporation_id,
        wallet_id,
        access_token,
        refresh_token,
        expires,
        scopes,
        now,
        now
    )
    .execute(app.get_db())
    .await?;

    Ok(())
}

pub async fn get_service_account_tokens(
    app: &crate::app::Application,
) -> Result<Option<(String, String, i64)>, Madness> {
    let result = sqlx::query!(
        "SELECT access_token, refresh_token, expires 
         FROM srp_service_account WHERE is_active = 1 LIMIT 1"
    )
    .fetch_optional(app.get_db())
    .await?;

    Ok(result.map(|r| (r.access_token, r.refresh_token, r.expires)))
}

pub async fn get_service_account_info(
    app: &crate::app::Application,
) -> Result<Option<SRPServiceAccount>, Madness> {
    let result = sqlx::query!(
        "SELECT character_id, character_name, corporation_id, wallet_id, 
                is_active, last_used, created_at
         FROM srp_service_account WHERE is_active = 1 LIMIT 1"
    )
    .fetch_optional(app.get_db())
    .await?;



    Ok(result.map(|r| SRPServiceAccount {
        character_id: r.character_id,
        character_name: r.character_name,
        corporation_id: r.corporation_id,
        wallet_id: r.wallet_id,
        is_active: r.is_active > 0,
        last_used: r.last_used,
        created_at: r.created_at,
    }))
}

pub async fn check_srp_validity(
    app: &crate::app::Application,
) -> Result<(), Madness> {
    let current_focus = crate::data::incursion::get_current_focus_status(app).await?;
    let focus_end_timestamp = current_focus.as_ref().and_then(|f| f.focus_end_timestamp);

    // If no focus has ended, nothing to void
    if focus_end_timestamp.is_none() {
        return Ok(());
    }

    let focus_end_time = focus_end_timestamp.unwrap();
    let now = chrono::Utc::now().timestamp();

    // Void per_focus payments that were made before the focus ended
    let result = sqlx::query!(
        "UPDATE srp_payments SET 
         expires_at = ?, 
         focus_voided_at = ?
         WHERE focus_voided_at IS NULL 
         AND coverage_type = 'per_focus'
         AND payment_date < ?",
        now,
        now,
        focus_end_time
    )
    .execute(app.get_db())
    .await?;

    info!("Voided {} per_focus SRP payments due to focus end", result.rows_affected());
    Ok(())
}

pub async fn get_all_srp_reports(
    app: &crate::app::Application,
) -> Result<Vec<SRPReport>, Madness> {
    let results = sqlx::query!(
        "SELECT r.killmail_id, r.killmail_link, r.submitted_at, r.loot_returned, r.description, 
                r.submitted_by_id, r.status, r.payout_amount, r.payout_date, r.srp_paid, r.reason,
                c.name as submitted_by_name, r.victim_character_name, r.victim_ship_type
         FROM srp_reports r
         LEFT JOIN `character` c ON r.submitted_by_id = c.id
         ORDER BY r.submitted_at DESC"
    )
    .fetch_all(app.get_db())
    .await?;

    Ok(results
        .into_iter()
        .map(|r| SRPReport {
            killmail_id: r.killmail_id,
            killmail_link: r.killmail_link,
            submitted_at: r.submitted_at,
            loot_returned: r.loot_returned > 0,
            description: r.description,
            submitted_by_id: r.submitted_by_id,
            submitted_by_name: r.submitted_by_name.unwrap_or_else(|| "Unknown".to_string()),
            status: r.status,
            payout_amount: r.payout_amount.map(|amount| amount.to_string().parse::<f64>().unwrap_or(0.0)),
            payout_date: r.payout_date,
            srp_paid: r.srp_paid.map(|val| val.to_string()),
            reason: r.reason,
            victim_character_name: r.victim_character_name,
            victim_ship_type: r.victim_ship_type,
        })
        .collect())
}

pub async fn get_srp_report_by_killmail_id(
    app: &crate::app::Application,
    killmail_id: i64,
) -> Result<Option<SRPReport>, Madness> {
    let result = sqlx::query!(
        "SELECT r.killmail_id, r.killmail_link, r.submitted_at, r.loot_returned, r.description, 
                r.submitted_by_id, r.status, r.payout_amount, r.payout_date, r.srp_paid, r.reason,
                c.name as submitted_by_name, r.victim_character_name, r.victim_ship_type
         FROM srp_reports r
         LEFT JOIN `character` c ON r.submitted_by_id = c.id
         WHERE r.killmail_id = ?",
        killmail_id
    )
    .fetch_optional(app.get_db())
    .await?;

    Ok(result.map(|r| SRPReport {
        killmail_id: r.killmail_id,
        killmail_link: r.killmail_link,
        submitted_at: r.submitted_at,
        loot_returned: r.loot_returned > 0,
        description: r.description,
        submitted_by_id: r.submitted_by_id,
        submitted_by_name: r.submitted_by_name.unwrap_or_else(|| "Unknown".to_string()),
        status: r.status,
        payout_amount: r.payout_amount.map(|amount| amount.to_string().parse::<f64>().unwrap_or(0.0)),
        payout_date: r.payout_date,
        srp_paid: r.srp_paid.map(|val| val.to_string()),
        reason: r.reason,
        victim_character_name: r.victim_character_name,
        victim_ship_type: r.victim_ship_type,
    }))
}

pub async fn get_killmail_data(
    app: &crate::app::Application,
    killmail_link: &str,
) -> Result<esi::KillmailData, Madness> {
    let (killmail_id, hash) = esi::extract_killmail_id_and_hash(killmail_link)
        .map_err(|e| Madness::BadRequest(e.to_string()))?;
    
    let killmail_data = app.esi_client.get_killmail(killmail_id, &hash).await
        .map_err(|e| Madness::BadRequest(format!("Failed to fetch killmail: {}", e)))?;
    
    Ok(killmail_data)
}

#[derive(Debug, Serialize)]
pub struct EnrichedKillmailData {
    pub killmail_id: i64,
    pub killmail_time: String,
    pub moon_id: Option<i64>,
    pub solar_system_id: i64,
    pub solar_system_name: Option<String>,
    pub war_id: Option<i64>,
    pub victim: EnrichedKillmailVictim,
    pub attackers: Vec<EnrichedKillmailAttacker>,
}

#[derive(Debug, Serialize)]
pub struct EnrichedKillmailVictim {
    pub alliance_id: Option<i64>,
    pub alliance_name: Option<String>,
    pub character_id: Option<i64>,
    pub character_name: Option<String>,
    pub corporation_id: Option<i64>,
    pub corporation_name: Option<String>,
    pub damage_taken: i64,
    pub items: Vec<EnrichedKillmailItem>,
    pub ship_type_id: i64,
    pub ship_name: String,
}

#[derive(Debug, Serialize)]
pub struct EnrichedKillmailAttacker {
    pub alliance_id: Option<i64>,
    pub alliance_name: Option<String>,
    pub character_id: Option<i64>,
    pub character_name: Option<String>,
    pub corporation_id: Option<i64>,
    pub corporation_name: Option<String>,
    pub damage_done: i64,
    pub final_blow: bool,
    pub security_status: f64,
    pub ship_type_id: Option<i64>,
    pub ship_name: Option<String>,
    pub weapon_type_id: Option<i64>,
    pub weapon_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EnrichedKillmailItem {
    pub flag: i32,
    pub item_type_id: i64,
    pub item_name: String,
    pub quantity_destroyed: Option<i64>,
    pub quantity_dropped: Option<i64>,
    pub singleton: i32,
}

pub async fn get_enriched_killmail_data(
    app: &crate::app::Application,
    killmail_link: &str,
) -> Result<EnrichedKillmailData, Madness> {
    let killmail_data = get_killmail_data(app, killmail_link).await?;
    
    // Collect all IDs that need name resolution
    let mut all_ids = Vec::new();
    
    // Victim IDs
    if let Some(character_id) = killmail_data.victim.character_id {
        all_ids.push(character_id);
    }
    if let Some(corporation_id) = killmail_data.victim.corporation_id {
        all_ids.push(corporation_id);
    }
    if let Some(alliance_id) = killmail_data.victim.alliance_id {
        all_ids.push(alliance_id);
    }
    all_ids.push(killmail_data.victim.ship_type_id);
    
    // Attacker IDs
    for attacker in &killmail_data.attackers {
        if let Some(character_id) = attacker.character_id {
            all_ids.push(character_id);
        }
        if let Some(corporation_id) = attacker.corporation_id {
            all_ids.push(corporation_id);
        }
        if let Some(alliance_id) = attacker.alliance_id {
            all_ids.push(alliance_id);
        }
        if let Some(ship_type_id) = attacker.ship_type_id {
            all_ids.push(ship_type_id);
        }
        if let Some(weapon_type_id) = attacker.weapon_type_id {
            all_ids.push(weapon_type_id);
        }
    }
    
    // Item IDs
    for item in &killmail_data.victim.items {
        all_ids.push(item.item_type_id);
    }
    
    // Solar system ID
    all_ids.push(killmail_data.solar_system_id);
    
    // Remove duplicates while preserving order
    let mut seen = std::collections::HashSet::new();
    all_ids.retain(|&id| seen.insert(id));
    
    // Get all names in one bulk request
    let names_map = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        app.esi_client.get_bulk_names(&all_ids)
    ).await
    .unwrap_or(Ok(std::collections::HashMap::new()))
    .unwrap_or_else(|_| std::collections::HashMap::new());
    
    // Extract victim names
    let mut victim_character_name = None;
    let mut victim_corporation_name = None;
    let mut victim_alliance_name = None;
    let mut victim_ship_name = "Unknown".to_string();
    
    if let Some(character_id) = killmail_data.victim.character_id {
        victim_character_name = names_map.get(&character_id).cloned();
    }
    
    if let Some(corporation_id) = killmail_data.victim.corporation_id {
        victim_corporation_name = names_map.get(&corporation_id).cloned();
    }
    
    if let Some(alliance_id) = killmail_data.victim.alliance_id {
        victim_alliance_name = names_map.get(&alliance_id).cloned();
    }
    
    victim_ship_name = names_map.get(&killmail_data.victim.ship_type_id)
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Get solar system name
    let solar_system_name = names_map.get(&killmail_data.solar_system_id).cloned();
    
    // Get attacker names
    let mut enriched_attackers = Vec::new();
    for attacker in &killmail_data.attackers {
        let character_name = attacker.character_id
            .and_then(|id| names_map.get(&id).cloned());
        let corporation_name = attacker.corporation_id
            .and_then(|id| names_map.get(&id).cloned());
        let alliance_name = attacker.alliance_id
            .and_then(|id| names_map.get(&id).cloned());
        let ship_name = attacker.ship_type_id
            .and_then(|id| names_map.get(&id).cloned());
        let weapon_name = attacker.weapon_type_id
            .and_then(|id| names_map.get(&id).cloned());
        
        enriched_attackers.push(EnrichedKillmailAttacker {
            alliance_id: attacker.alliance_id,
            alliance_name,
            character_id: attacker.character_id,
            character_name,
            corporation_id: attacker.corporation_id,
            corporation_name,
            damage_done: attacker.damage_done,
            final_blow: attacker.final_blow,
            security_status: attacker.security_status,
            ship_type_id: attacker.ship_type_id,
            ship_name,
            weapon_type_id: attacker.weapon_type_id,
            weapon_name,
        });
    }
    
    // Get item names
    let mut enriched_items = Vec::new();
    for item in &killmail_data.victim.items {
        let item_name = names_map.get(&item.item_type_id)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());
        
        enriched_items.push(EnrichedKillmailItem {
            flag: item.flag,
            item_type_id: item.item_type_id,
            item_name,
            quantity_destroyed: item.quantity_destroyed,
            quantity_dropped: item.quantity_dropped,
            singleton: item.singleton,
        });
    }
    
    Ok(EnrichedKillmailData {
        killmail_id: killmail_data.killmail_id,
        killmail_time: killmail_data.killmail_time,
        moon_id: killmail_data.moon_id,
        solar_system_id: killmail_data.solar_system_id,
        solar_system_name,
        war_id: killmail_data.war_id,
        victim: EnrichedKillmailVictim {
            alliance_id: killmail_data.victim.alliance_id,
            alliance_name: victim_alliance_name,
            character_id: killmail_data.victim.character_id,
            character_name: victim_character_name,
            corporation_id: killmail_data.victim.corporation_id,
            corporation_name: victim_corporation_name,
            damage_taken: killmail_data.victim.damage_taken,
            items: enriched_items,
            ship_type_id: killmail_data.victim.ship_type_id,
            ship_name: victim_ship_name,
        },
        attackers: enriched_attackers,
    })
}

pub async fn check_fleet_membership_at_time(
    app: &crate::app::Application,
    character_id: i64,
    timestamp: i64,
) -> Result<bool, Madness> {
    let result = sqlx::query!(
        "SELECT character_id FROM fleet_activity WHERE character_id = ? AND first_seen <= ? AND last_seen >= ? LIMIT 1",
        character_id,
        timestamp,
        timestamp
    )
    .fetch_optional(app.get_db())
    .await?;

    Ok(result.is_some())
}

pub async fn check_srp_status_at_time(
    app: &crate::app::Application,
    character_name: &str,
    timestamp: i64,
) -> Result<bool, Madness> {
    // Check if the character has valid SRP coverage at the specified time
    let result = sqlx::query!(
        "SELECT character_name FROM srp_payments 
         WHERE character_name = ? 
         AND payment_date <= ? 
         AND expires_at >= ?
         LIMIT 1",
        character_name,
        timestamp,
        timestamp
    )
    .fetch_optional(app.get_db())
    .await?;

    Ok(result.is_some())
}

pub async fn calculate_srp_appraisal(
    app: &crate::app::Application,
    destroyed_items: &[String],
) -> Result<(f64, Vec<serde_json::Value>), Madness> {
    if destroyed_items.is_empty() {
        return Ok((0.0, Vec::new()));
    }

    // Format items for Janice API (ItemName Quantity format)
    let items_text = destroyed_items.join("\n");
    
    // Make request to Janice API
    let client = reqwest::Client::new();
    let url = "https://janice.e-351.com/api/rest/v2/appraisal?market=2&pricing=sell&persist=false&compactize=true&pricePercentage=1";
    
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .header("accept", "application/json")
        .header("X-ApiKey", &app.config.janice.api_key)
        .body(items_text)
        .send()
        .await
        .map_err(|e| Madness::BadRequest(format!("Failed to call Janice API: {}", e)))?;

    let status = response.status();
    let response_text = response.text().await
        .map_err(|e| Madness::BadRequest(format!("Failed to read Janice API response: {}", e)))?;

    if !status.is_success() {
        return Err(Madness::BadRequest(format!(
            "Janice API returned error: {} - Body: {}",
            status, response_text
        )));
    }

    // Parse the successful response
    let appraisal_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| Madness::BadRequest(format!("Failed to parse Janice API response: {}", e)))?;

    // Extract the total value and items from the response
    let total_value = appraisal_data
        .get("effectivePrices")
        .and_then(|v| v.get("totalSellPrice"))
        .and_then(|v| v.as_f64())
        .ok_or_else(|| Madness::BadRequest("Invalid response format from Janice API - missing totalSellPrice".to_string()))?;

    let items = appraisal_data
        .get("items")
        .and_then(|v| v.as_array())
        .map(|arr| arr.clone())
        .unwrap_or_else(Vec::new);

    Ok((total_value, items))
}

pub async fn approve_srp_report(
    app: &crate::app::Application,
    killmail_id: i64,
    admin_character_id: i64,
    window_character_id: Option<i64>,
    payout_amount: f64,
) -> Result<(), Madness> {
    // First, get the SRP report to get the killmail link
    let report = sqlx::query!(
        "SELECT killmail_link FROM srp_reports WHERE killmail_id = ?",
        killmail_id
    )
    .fetch_optional(app.get_db())
    .await?;

    let report = report.ok_or_else(|| Madness::NotFound("SRP report not found"))?;

    // Update the SRP report status and payout amount
    let result = sqlx::query!(
        "UPDATE srp_reports 
         SET status = 'approved', payout_amount = ? 
         WHERE killmail_id = ?",
        payout_amount,
        killmail_id
    )
    .execute(app.get_db())
    .await?;

    if result.rows_affected() == 0 {
        return Err(Madness::NotFound("SRP report not found"));
    }

    Ok(())
}

pub async fn deny_srp_report(
    app: &crate::app::Application,
    killmail_id: i64,
    reason: &str,
) -> Result<(), Madness> {
    eprintln!("Deny SRP report data function called for killmail_id: {}, reason: {}", killmail_id, reason);
    
    let result = sqlx::query!(
        "UPDATE srp_reports 
         SET status = 'rejected', reason = ? 
         WHERE killmail_id = ?",
        reason,
        killmail_id
    )
    .execute(app.get_db())
    .await?;

    eprintln!("Deny SRP report SQL result: rows_affected = {}", result.rows_affected());

    if result.rows_affected() == 0 {
        return Err(Madness::NotFound("SRP report not found"));
    }

    Ok(())
}

async fn get_srp_coverage_details(
    app: &crate::app::Application,
    killmail_data: &KillmailData,
) -> Result<String, Madness> {
    if let Some(character_id) = killmail_data.victim.character_id {
        let had_coverage = check_srp_coverage_for_character(app, character_id, killmail_data.killmail_time.clone()).await?;
        
        if had_coverage {
            // Get detailed SRP payment information
            let character_name = sqlx::query!(
                "SELECT name FROM `character` WHERE id = ?",
                character_id
            )
            .fetch_optional(app.get_db())
            .await?;

            if let Some(character) = character_name {
                // Helper function to get SRP payment details for a character
                async fn get_srp_payment_details(app: &crate::app::Application, character_name: &str, killmail_time: &str) -> Result<Option<serde_json::Value>, Madness> {
                    let timestamp = chrono::DateTime::parse_from_rfc3339(killmail_time)
                        .map_err(|_| Madness::BadRequest("Invalid killmail time format".to_string()))?
                        .timestamp();

                    let result = sqlx::query!(
                        "SELECT payment_amount, payment_date FROM srp_payments 
                         WHERE character_name = ? 
                         AND payment_date <= ? 
                         AND expires_at > ? 
                         AND focus_voided_at IS NULL
                         ORDER BY payment_date DESC 
                         LIMIT 1",
                        character_name,
                        timestamp,
                        timestamp
                    )
                    .fetch_optional(app.get_db())
                    .await?;

                    if let Some(payment) = result {
                        let payment_date = chrono::DateTime::<chrono::Utc>::from_utc(
                            chrono::NaiveDateTime::from_timestamp_opt(payment.payment_date, 0).unwrap(),
                            chrono::Utc
                        ).format("%Y-%m-%d %H:%M UTC").to_string();

                        Ok(Some(serde_json::json!({
                            "had_coverage": true,
                            "payment_date": payment_date,
                            "payment_amount": payment.payment_amount.to_string(),
                            "coverage_character": character_name,
                            "coverage_type": "direct"
                        })))
                    } else {
                        Ok(None)
                    }
                }

                // Check direct character first
                if let Some(details) = get_srp_payment_details(app, &character.name, &killmail_data.killmail_time).await? {
                    return Ok(details.to_string());
                }

                // Check main character (if this is an alt)
                let main_character = sqlx::query!(
                    "SELECT account_id FROM alt_character WHERE alt_id = ?",
                    character_id
                )
                .fetch_optional(app.get_db())
                .await?;

                if let Some(main) = main_character {
                    let main_name = sqlx::query!(
                        "SELECT name FROM `character` WHERE id = ?",
                        main.account_id
                    )
                    .fetch_optional(app.get_db())
                    .await?;

                    if let Some(main_char) = main_name {
                        if let Some(details) = get_srp_payment_details(app, &main_char.name, &killmail_data.killmail_time).await? {
                            let mut details_obj = details.as_object().unwrap().clone();
                            details_obj.insert("coverage_type".to_string(), serde_json::Value::String("main".to_string()));
                            details_obj.insert("coverage_character".to_string(), serde_json::Value::String(main_char.name));
                            return Ok(serde_json::Value::Object(details_obj).to_string());
                        }

                        // Check alt characters (if this is a main)
                        let alt_characters = sqlx::query!(
                            "SELECT alt_id FROM alt_character WHERE account_id = ?",
                            character_id
                        )
                        .fetch_all(app.get_db())
                        .await?;

                        for alt in alt_characters {
                            let alt_name = sqlx::query!(
                                "SELECT name FROM `character` WHERE id = ?",
                                alt.alt_id
                            )
                            .fetch_optional(app.get_db())
                            .await?;

                            if let Some(alt_char) = alt_name {
                                if let Some(details) = get_srp_payment_details(app, &alt_char.name, &killmail_data.killmail_time).await? {
                                    let mut details_obj = details.as_object().unwrap().clone();
                                    details_obj.insert("coverage_type".to_string(), serde_json::Value::String("alt".to_string()));
                                    details_obj.insert("coverage_character".to_string(), serde_json::Value::String(alt_char.name));
                                    return Ok(serde_json::Value::Object(details_obj).to_string());
                                }
                            }
                        }
                    }
                } else {
                    // This is a main character, check alts
                    let alt_characters = sqlx::query!(
                        "SELECT alt_id FROM alt_character WHERE account_id = ?",
                        character_id
                    )
                    .fetch_all(app.get_db())
                    .await?;

                    for alt in alt_characters {
                        let alt_name = sqlx::query!(
                            "SELECT name FROM `character` WHERE id = ?",
                            alt.alt_id
                        )
                        .fetch_optional(app.get_db())
                        .await?;

                        if let Some(alt_char) = alt_name {
                            if let Some(details) = get_srp_payment_details(app, &alt_char.name, &killmail_data.killmail_time).await? {
                                let mut details_obj = details.as_object().unwrap().clone();
                                details_obj.insert("coverage_type".to_string(), serde_json::Value::String("alt".to_string()));
                                details_obj.insert("coverage_character".to_string(), serde_json::Value::String(alt_char.name));
                                return Ok(serde_json::Value::Object(details_obj).to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(serde_json::json!({"had_coverage": false}).to_string())
}

async fn get_fleet_composition_at_time(
    app: &crate::app::Application,
    killmail_time: &str,
    victim_character_id: Option<i64>,
) -> Result<Option<serde_json::Value>, Madness> {
    // Parse the killmail time
    let killmail_timestamp = chrono::DateTime::parse_from_rfc3339(killmail_time)
        .map_err(|_| Madness::BadRequest("Invalid killmail time format".to_string()))?
        .timestamp();

    // If we have a victim character ID, verify they were in the fleet at the time
    if let Some(victim_id) = victim_character_id {
        let victim_in_fleet = sqlx::query!(
            "SELECT id FROM fleet_activity 
             WHERE character_id = ? 
             AND first_seen <= ? 
             AND last_seen >= ?
             AND has_left = 0",
            victim_id,
            killmail_timestamp,
            killmail_timestamp
        )
        .fetch_optional(app.get_db())
        .await?;

        if victim_in_fleet.is_none() {
            // Victim was not in any fleet at the time of death
            return Ok(None);
        }
    }

    // Find any fleet that had members active at the time of the killmail
    let fleet_members = sqlx::query!(
        "SELECT hull FROM fleet_activity 
         WHERE first_seen <= ? 
         AND last_seen >= ?
         AND has_left = 0",
        killmail_timestamp,
        killmail_timestamp
    )
    .fetch_all(app.get_db())
    .await?;

    if !fleet_members.is_empty() {
        // Count ships by hull type
        let mut hull_counts = std::collections::HashMap::new();
        for member in fleet_members {
            *hull_counts.entry(member.hull).or_insert(0) += 1;
        }

        // Convert hull IDs to ship names and create JSON
        let mut fleet_comp = serde_json::Map::new();
        for (hull_id, count) in hull_counts {
            // Try to get ship name from ESI with timeout
            match tokio::time::timeout(
                std::time::Duration::from_secs(2),
                app.esi_client.get_type_name(hull_id.into())
            ).await {
                Ok(Ok(ship_name)) => {
                    fleet_comp.insert(ship_name, serde_json::Value::Number(count.into()));
                },
                _ => {
                    // Fallback to hull ID if name lookup fails
                    fleet_comp.insert(format!("Ship_{}", hull_id), serde_json::Value::Number(count.into()));
                }
            }
        }

        return Ok(Some(serde_json::Value::Object(fleet_comp)));
    }

    // No fleet members found at killmail time
    Ok(None)
}

pub async fn submit_srp_report(
    app: &crate::app::Application,
    killmail_link: &str,
    description: &str,
    loot_returned: bool,
    submitted_by_id: i64,
) -> Result<(), Madness> {
    // Extract killmail ID and hash from the link
    let (killmail_id, _hash) = esi::extract_killmail_id_and_hash(killmail_link)
        .map_err(|e| Madness::BadRequest(format!("Invalid killmail link: {}", e)))?;

    // Get killmail data from ESI
    let killmail_data = get_killmail_data(app, killmail_link).await?;

    // Extract victim information
    let victim_character_name = if let Some(character_id) = killmail_data.victim.character_id {
        // Try to get character name from database first
        let db_name = sqlx::query!(
            "SELECT name FROM `character` WHERE id = ?",
            character_id
        )
        .fetch_optional(app.get_db())
        .await?;

        if let Some(name) = db_name {
            Some(name.name)
        } else {
            // Fallback to ESI lookup with timeout
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                app.esi_client.get_character_name(character_id)
            ).await {
                Ok(Ok(name)) => Some(name),
                _ => Some("Unknown".to_string())
            }
        }
    } else {
        None // NPC kill
    };

    // Get ship type name
    let victim_ship_type = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        app.esi_client.get_type_name(killmail_data.victim.ship_type_id)
    ).await {
        Ok(Ok(name)) => name,
        _ => "Unknown".to_string()
    };

    // Check SRP coverage for the victim and get detailed information
    let srp_paid_data = get_srp_coverage_details(app, &killmail_data).await?;

    // Get fleet composition for the time of the killmail
    let fleet_comp = get_fleet_composition_at_time(app, &killmail_data.killmail_time, killmail_data.victim.character_id).await?;

    // Check if SRP report already exists
    let existing_report = sqlx::query!(
        "SELECT killmail_id FROM srp_reports WHERE killmail_id = ?",
        killmail_id
    )
    .fetch_optional(app.get_db())
    .await?;

    if existing_report.is_some() {
        return Err(Madness::BadRequest("An SRP report for this killmail already exists. Use the update button to modify existing reports.".to_string()));
    }

    // Insert new report
    sqlx::query!(
        "INSERT INTO srp_reports 
         (killmail_id, killmail_link, submitted_at, loot_returned, description, 
          submitted_by_id, status, victim_character_name, victim_ship_type, srp_paid, fleet_comp)
         VALUES (?, ?, ?, ?, ?, ?, 'pending', ?, ?, ?, ?)",
        killmail_id,
        killmail_link,
        chrono::Utc::now().timestamp(),
        loot_returned,
        description,
        submitted_by_id,
        victim_character_name,
        victim_ship_type,
        srp_paid_data,
        fleet_comp
    )
    .execute(app.get_db())
    .await?;

    Ok(())
}

async fn check_srp_coverage_for_character(
    app: &crate::app::Application,
    character_id: i64,
    killmail_time: String,
) -> Result<bool, Madness> {
    // Parse the killmail time
    let killmail_timestamp = chrono::DateTime::parse_from_rfc3339(&killmail_time)
        .map_err(|_| Madness::BadRequest("Invalid killmail time format".to_string()))?
        .timestamp();

    // Check direct character SRP coverage
    let direct_coverage = sqlx::query!(
        "SELECT character_name FROM srp_payments 
         WHERE character_name = (SELECT name FROM `character` WHERE id = ?) 
         AND payment_date <= ? AND expires_at > ? AND focus_voided_at IS NULL",
        character_id,
        killmail_timestamp,
        killmail_timestamp
    )
    .fetch_optional(app.get_db())
    .await?;

    if direct_coverage.is_some() {
        return Ok(true);
    }

    // Check main character coverage (if this is an alt)
    let main_character = sqlx::query!(
        "SELECT account_id FROM alt_character WHERE alt_id = ?",
        character_id
    )
    .fetch_optional(app.get_db())
    .await?;

    if let Some(main) = main_character {
        let main_coverage = sqlx::query!(
            "SELECT character_name FROM srp_payments 
             WHERE character_name = (SELECT name FROM `character` WHERE id = ?) 
             AND payment_date <= ? AND expires_at > ? AND focus_voided_at IS NULL",
            main.account_id,
            killmail_timestamp,
            killmail_timestamp
        )
        .fetch_optional(app.get_db())
        .await?;

        if main_coverage.is_some() {
            return Ok(true);
        }
    }

    // Check alt character coverage (if this is a main)
    let alt_characters = sqlx::query!(
        "SELECT alt_id FROM alt_character WHERE account_id = ?",
        character_id
    )
    .fetch_all(app.get_db())
    .await?;

    for alt in alt_characters {
        let alt_coverage = sqlx::query!(
            "SELECT character_name FROM srp_payments 
             WHERE character_name = (SELECT name FROM `character` WHERE id = ?) 
             AND payment_date <= ? AND expires_at > ? AND focus_voided_at IS NULL",
            alt.alt_id,
            killmail_timestamp,
            killmail_timestamp
        )
        .fetch_optional(app.get_db())
        .await?;

        if alt_coverage.is_some() {
            return Ok(true);
        }
    }

    Ok(false)
}

pub async fn update_srp_report(
    app: &crate::app::Application,
    killmail_id: i64,
    description: &str,
    loot_returned: bool,
) -> Result<(), Madness> {
    // Check if SRP report exists and get killmail link for fleet composition
    let existing_report = sqlx::query!(
        "SELECT killmail_link FROM srp_reports WHERE killmail_id = ?",
        killmail_id
    )
    .fetch_optional(app.get_db())
    .await?;

    if existing_report.is_none() {
        return Err(Madness::NotFound("SRP report not found"));
    }

    let report = existing_report.unwrap();
    
    // Get killmail data to get the time for fleet composition
    let killmail_data = get_killmail_data(app, &report.killmail_link).await?;
    
    // Get fleet composition for the time of the killmail
    let fleet_comp = get_fleet_composition_at_time(app, &killmail_data.killmail_time, killmail_data.victim.character_id).await?;

    // Update the report
    sqlx::query!(
        "UPDATE srp_reports 
         SET description = ?, loot_returned = ?, fleet_comp = ?
         WHERE killmail_id = ?",
        description,
        loot_returned,
        fleet_comp,
        killmail_id
    )
    .execute(app.get_db())
    .await?;

    Ok(())
}

pub async fn get_srp_reports_for_pilot(
    app: &crate::app::Application,
    character_id: i64,
) -> Result<Vec<SRPReport>, Madness> {
    // Get the character name
    let character_name = sqlx::query!(
        "SELECT name FROM `character` WHERE id = ?",
        character_id
    )
    .fetch_optional(app.get_db())
    .await?;

    if character_name.is_none() {
        return Ok(Vec::new());
    }

    let character_name = character_name.unwrap().name;

    // Get all alt character IDs for this character
    let alt_characters = sqlx::query!(
        "SELECT alt_id FROM alt_character WHERE account_id = ?",
        character_id
    )
    .fetch_all(app.get_db())
    .await?;

    let main_characters = sqlx::query!(
        "SELECT account_id FROM alt_character WHERE alt_id = ?",
        character_id
    )
    .fetch_all(app.get_db())
    .await?;

    // Build a list of all character IDs to check (main + alts)
    let mut all_character_ids = Vec::new();
    all_character_ids.push(character_id);
    
    for alt in alt_characters {
        all_character_ids.push(alt.alt_id);
    }
    
    for main in main_characters {
        all_character_ids.push(main.account_id);
    }

    // Get character names for all these IDs
    let character_names = if all_character_ids.is_empty() {
        Vec::new()
    } else {
        let placeholders = all_character_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!("SELECT id, name FROM `character` WHERE id IN ({})", placeholders);
        
        let mut query_builder = sqlx::query_as::<_, (i64, String)>(&query);
        for id in &all_character_ids {
            query_builder = query_builder.bind(id);
        }
        query_builder.fetch_all(app.get_db()).await?
    };

    // Build the WHERE clause for victim character names
    let victim_names: Vec<String> = character_names.iter().map(|c| c.1.clone()).collect();
    
    // Use a simpler approach with raw query
    if victim_names.is_empty() {
        return Ok(Vec::new());
    }
    
    let placeholders = victim_names.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    
    let query = format!(
        "SELECT r.killmail_id, r.killmail_link, r.submitted_at, r.loot_returned, 
                r.description, r.submitted_by_id, r.status, r.payout_date, 
                r.payout_amount, r.reason, r.victim_character_name, r.victim_ship_type, r.srp_paid,
                c.name as submitted_by_name
         FROM srp_reports r
         LEFT JOIN `character` c ON r.submitted_by_id = c.id
         WHERE r.victim_character_name IN ({})
         ORDER BY r.submitted_at DESC",
        placeholders
    );

    let mut query_builder = sqlx::query(&query);
    
    // Bind all the character names
    for name in &victim_names {
        query_builder = query_builder.bind(name);
    }

    let rows = query_builder.fetch_all(app.get_db()).await?;
    
    let mut results = Vec::new();
    for row in rows {
        results.push(SRPReport {
            killmail_id: row.try_get("killmail_id")?,
            killmail_link: row.try_get("killmail_link")?,
            submitted_at: row.try_get("submitted_at")?,
            loot_returned: row.try_get("loot_returned")?,
            description: row.try_get("description")?,
            submitted_by_id: row.try_get("submitted_by_id")?,
            status: row.try_get("status")?,
            payout_date: row.try_get("payout_date")?,
            payout_amount: row.try_get("payout_amount")?,
            reason: row.try_get("reason")?,
            victim_character_name: row.try_get("victim_character_name")?,
            victim_ship_type: row.try_get("victim_ship_type")?,
            srp_paid: row.try_get("srp_paid")?,
            submitted_by_name: row.try_get("submitted_by_name")?,
        });
    }
    
    Ok(results)
}
