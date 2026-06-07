use crate::app::Application;
use crate::config::Config;
use crate::core::discord::{AllowedMentions, DiscordError, Embed, EmbedField, WebhookPayload};
use sqlx::Row;

struct SrpReportNotifyData {
    killmail_id: i64,
    status: String,
    victim_character_name: Option<String>,
    victim_ship_type: Option<String>,
    submitted_by_name: Option<String>,
    reason: Option<String>,
    discord_message_id: Option<i64>,
}

pub fn spawn_notify_srp_report(app: &Application, killmail_id: i64) {
    let db = app.get_db().clone();
    let config = app.config.clone();
    let discord_client = app.discord_client.clone();

    tokio::spawn(async move {
        if let Err(e) = notify_srp_report(db, config, discord_client, killmail_id).await {
            error!(
                "Failed to notify Discord for SRP report {}: {:#?}",
                killmail_id, e
            );
        }
    });
}

async fn notify_srp_report(
    db: crate::DB,
    config: Config,
    discord_client: crate::core::discord::DiscordWebhookClient,
    killmail_id: i64,
) -> Result<(), DiscordError> {
    if !config.discord.enabled || config.discord.srp_webhook_url.is_empty() {
        return Ok(());
    }

    let report = load_report(&db, killmail_id)
        .await
        .map_err(|e| DiscordError::ApiError(e.to_string()))?;

    let webhook_url = config.discord.srp_webhook_url.as_str();
    let site_url = site_url(&config);

    if let Some(message_id) = report.discord_message_id {
        let payload = build_payload(&report, &site_url, false, &config.discord.srp_ping_role_id);
        discord_client
            .edit_message(webhook_url, message_id as u64, &payload)
            .await?;
        return Ok(());
    }

    let payload = build_payload(&report, &site_url, true, &config.discord.srp_ping_role_id);
    let message_id = discord_client
        .send_message(webhook_url, &payload)
        .await?;

    sqlx::query(
        "UPDATE srp_reports SET discord_message_id = ? WHERE killmail_id = ?",
    )
    .bind(message_id as i64)
    .bind(killmail_id)
    .execute(&db)
    .await
    .map_err(|e| DiscordError::ApiError(e.to_string()))?;

    Ok(())
}

async fn load_report(db: &crate::DB, killmail_id: i64) -> Result<SrpReportNotifyData, sqlx::Error> {
    let row = sqlx::query(
        "SELECT r.killmail_id, r.status, r.victim_character_name,
                r.victim_ship_type, r.reason, r.discord_message_id,
                c.name AS submitted_by_name
         FROM srp_reports r
         LEFT JOIN `character` c ON r.submitted_by_id = c.id
         WHERE r.killmail_id = ?",
    )
    .bind(killmail_id)
    .fetch_one(db)
    .await?;

    Ok(SrpReportNotifyData {
        killmail_id: row.try_get("killmail_id")?,
        status: row.try_get("status")?,
        victim_character_name: row.try_get("victim_character_name")?,
        victim_ship_type: row.try_get("victim_ship_type")?,
        submitted_by_name: row.try_get("submitted_by_name")?,
        reason: row.try_get("reason")?,
        discord_message_id: row.try_get("discord_message_id")?,
    })
}

fn build_payload(
    report: &SrpReportNotifyData,
    site_url: &str,
    ping_role: bool,
    role_id: &str,
) -> WebhookPayload {
    let status_label = status_label(&report.status);
    let mut fields = vec![
        embed_field("Status", status_label),
        embed_field(
            "Victim",
            report
                .victim_character_name
                .as_deref()
                .unwrap_or("Unknown"),
        ),
        embed_field(
            "Ship",
            report.victim_ship_type.as_deref().unwrap_or("Unknown"),
        ),
        embed_field(
            "Submitted by",
            report.submitted_by_name.as_deref().unwrap_or("Unknown"),
        ),
    ];

    if report.status == "rejected" {
        if let Some(reason) = &report.reason {
            if !reason.is_empty() {
                fields.push(embed_field("Reason", reason));
            }
        }
    }

    let report_url = format!(
        "{}/srp-report-detail?id={}",
        site_url, report.killmail_id
    );

    let (content, allowed_mentions) = if ping_role && !role_id.is_empty() {
        (
            Some(format!("<@&{}>", role_id)),
            Some(AllowedMentions {
                parse: Some(vec![]),
                roles: Some(vec![role_id.to_string()]),
            }),
        )
    } else {
        (None, None)
    };

    WebhookPayload {
        content,
        embeds: Some(vec![Embed {
            title: format!("SRP Report — {}", status_label),
            url: Some(report_url),
            description: None,
            color: status_color(&report.status),
            fields: Some(fields),
        }]),
        allowed_mentions,
    }
}

fn site_url(config: &Config) -> String {
    if !config.discord.site_url.is_empty() {
        return normalize_site_url(&config.discord.site_url);
    }

    let derived = config
        .esi
        .url
        .rsplit_once('/')
        .map(|(base, _)| base.to_string())
        .unwrap_or_else(|| config.esi.url.clone());

    normalize_site_url(&derived)
}

fn normalize_site_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{}", trimmed)
    }
}

fn embed_field(name: &str, value: &str) -> EmbedField {
    EmbedField {
        name: name.to_string(),
        value: value.to_string(),
        inline: Some(true),
    }
}

fn status_label(status: &str) -> &'static str {
    match status {
        "approved" => "Approved",
        "rejected" => "Rejected",
        "paid" => "Paid",
        _ => "Pending",
    }
}

fn status_color(status: &str) -> u32 {
    match status {
        "approved" => 0x57F287,
        "rejected" => 0xED4245,
        "paid" => 0x5865F2,
        _ => 0xFEE75C,
    }
}
