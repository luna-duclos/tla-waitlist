use crate::core::esi::ESIScope;
use crate::util::madness::Madness;
use serde::{Deserialize, Serialize};

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
        // Only process incoming payments (positive amounts)
        if entry.amount <= 0.0 {
            continue;
        }

        // Parse the entry date
        let entry_date: chrono::DateTime<chrono::Utc> = chrono::DateTime::parse_from_rfc3339(&entry.date)
            .map_err(|_| Madness::BadRequest("Invalid date format".to_string()))?
            .into(); // Convert to UTC

        // Only process entries from the last 8 days
        if entry_date < eight_days_ago {
            continue;
        }

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
                // Handle corporate transfers
                let parts: Vec<&str> = entry.description.split(" transferred cash from ").collect();
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
