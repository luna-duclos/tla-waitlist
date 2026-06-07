use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DiscordWebhookClient {
    http: reqwest::Client,
}

#[derive(thiserror::Error, Debug)]
pub enum DiscordError {
    #[error("HTTP error while contacting Discord")]
    HTTPError(#[from] reqwest::Error),
    #[error("Discord API error: {0}")]
    ApiError(String),
}

#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_mentions: Option<AllowedMentions>,
}

#[derive(Debug, Serialize)]
pub struct AllowedMentions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct Embed {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub color: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<EmbedField>>,
}

#[derive(Debug, Serialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct WebhookMessageResponse {
    id: String,
}

impl DiscordWebhookClient {
    pub fn new() -> DiscordWebhookClient {
        DiscordWebhookClient {
            http: reqwest::Client::new(),
        }
    }

    pub async fn send_message(
        &self,
        webhook_url: &str,
        payload: &WebhookPayload,
    ) -> Result<u64, DiscordError> {
        let url = webhook_url_with_wait(webhook_url);

        let response = self.http.post(&url).json(payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DiscordError::ApiError(format!("{}: {}", status, body)));
        }

        let body: WebhookMessageResponse = response.json().await?;
        body.id
            .parse()
            .map_err(|_| DiscordError::ApiError(format!("Invalid message id: {}", body.id)))
    }

    pub async fn edit_message(
        &self,
        webhook_url: &str,
        message_id: u64,
        payload: &WebhookPayload,
    ) -> Result<(), DiscordError> {
        let base_url = webhook_base_url(webhook_url);
        let url = format!("{}/messages/{}", base_url, message_id);

        let response = self.http.patch(&url).json(payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DiscordError::ApiError(format!("{}: {}", status, body)));
        }

        Ok(())
    }
}

fn webhook_base_url(webhook_url: &str) -> &str {
    webhook_url.split('?').next().unwrap_or(webhook_url).trim_end_matches('/')
}

fn webhook_url_with_wait(webhook_url: &str) -> String {
    if webhook_url.contains('?') {
        format!("{}&wait=true", webhook_url)
    } else {
        format!("{}?wait=true", webhook_url)
    }
}
