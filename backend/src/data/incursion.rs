use crate::util::madness::Madness;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct IncursionFocus {
    pub id: i32,
    pub current_focus_constellation_id: Option<i64>,
    pub current_focus_constellation_name: Option<String>,
    pub last_check_timestamp: i64,
    pub focus_active: bool,
    pub focus_end_timestamp: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

// Cache for constellation names to reduce ESI calls
static mut CONSTELLATION_CACHE: Option<HashMap<i64, String>> = None;

pub fn get_constellation_cache() -> &'static mut HashMap<i64, String> {
    unsafe {
        if CONSTELLATION_CACHE.is_none() {
            CONSTELLATION_CACHE = Some(HashMap::new());
        }
        CONSTELLATION_CACHE.as_mut().unwrap()
    }
}

pub async fn get_current_focus_status(
    app: &crate::app::Application,
) -> Result<Option<IncursionFocus>, Madness> {
    let result = sqlx::query!(
        "SELECT id, current_focus_constellation_id, current_focus_constellation_name, 
                last_check_timestamp, focus_active, focus_end_timestamp, created_at, updated_at
         FROM incursion_focus ORDER BY id DESC LIMIT 1"
    )
    .fetch_optional(app.get_db())
    .await?;

    Ok(result.map(|r| IncursionFocus {
        id: r.id,
        current_focus_constellation_id: r.current_focus_constellation_id,
        current_focus_constellation_name: r.current_focus_constellation_name,
        last_check_timestamp: r.last_check_timestamp,
        focus_active: r.focus_active > 0,
        focus_end_timestamp: r.focus_end_timestamp,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }))
}

pub async fn update_focus_status(
    app: &crate::app::Application,
    constellation_id: Option<i64>,
    constellation_name: Option<String>,
    focus_active: bool,
    focus_end_timestamp: Option<i64>,
) -> Result<(), Madness> {
    let now = chrono::Utc::now().timestamp();

    sqlx::query!(
        "UPDATE incursion_focus SET 
         current_focus_constellation_id = ?, 
         current_focus_constellation_name = ?, 
         last_check_timestamp = ?, 
         focus_active = ?, 
         focus_end_timestamp = ?,
         updated_at = ?
         WHERE id = (SELECT id FROM (SELECT id FROM incursion_focus ORDER BY id DESC LIMIT 1) as sub)",
        constellation_id,
        constellation_name,
        now,
        focus_active,
        focus_end_timestamp,
        now
    )
    .execute(app.get_db())
    .await?;

    Ok(())
}

pub async fn detect_highsec_focus(
    app: &crate::app::Application,
) -> Result<Option<(i64, String)>, Madness> {
    // Get all active incursions
    let incursions = app.esi_client.get_incursions().await
        .map_err(|e| Madness::BadRequest(format!("Failed to fetch incursions: {:?}", e)))?;

    let constellation_cache = get_constellation_cache();

    for incursion in incursions {
        // Get staging system info to check security status
        let system_info = app.esi_client.get_solar_system_info(incursion.staging_solar_system_id).await
            .map_err(|e| Madness::BadRequest(format!("Failed to fetch system info: {:?}", e)))?;

        // Check if this is a highsec focus (security status >= 0.5)
        if system_info.security_status >= 0.5 {
            // Get constellation name (use cache if available)
            let constellation_name = if let Some(cached_name) = constellation_cache.get(&incursion.constellation_id) {
                cached_name.clone()
            } else {
                let constellation_info = app.esi_client.get_constellation_info(incursion.constellation_id).await
                    .map_err(|e| Madness::BadRequest(format!("Failed to fetch constellation info: {:?}", e)))?;
                
                // Cache the constellation name
                constellation_cache.insert(incursion.constellation_id, constellation_info.name.clone());
                constellation_info.name
            };

            return Ok(Some((incursion.constellation_id, constellation_name)));
        }
    }

    Ok(None)
}



pub async fn check_and_update_focus(
    app: &crate::app::Application,
) -> Result<(), Madness> {
    let current_focus = get_current_focus_status(app).await?;
    let current_constellation_id = current_focus.as_ref().and_then(|f| f.current_focus_constellation_id);
    let current_focus_active = current_focus.as_ref().map(|f| f.focus_active).unwrap_or(false);

    // Detect current highsec focus
    let detected_focus = detect_highsec_focus(app).await?;
    let new_constellation_id = detected_focus.as_ref().map(|(id, _)| *id);
    let new_focus_active = detected_focus.is_some();

    // Check if focus has ended (transition from active to inactive)
    let focus_ended = current_focus_active && !new_focus_active;
    let focus_end_timestamp = if focus_ended {
        Some(chrono::Utc::now().timestamp())
    } else {
        current_focus.as_ref().and_then(|f| f.focus_end_timestamp)
    };

    // Check SRP validity based on focus end
    crate::data::srp::check_srp_validity(app).await?;

    // Update focus status
    let constellation_name = detected_focus.as_ref().map(|(_, name)| name.clone());
    update_focus_status(app, new_constellation_id, constellation_name, new_focus_active, focus_end_timestamp).await?;

    // Log focus changes
    if current_constellation_id != new_constellation_id {
        if let Some((_, name)) = detected_focus {
            info!("New highsec incursion focus detected: {} (ID: {})", name, new_constellation_id.unwrap());
        } else {
            info!("Highsec incursion focus has ended");
        }
    }

    Ok(())
}
