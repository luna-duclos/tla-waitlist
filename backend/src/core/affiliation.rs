use serde::Deserialize;
use std::sync::Arc;

use crate::util::madness::Madness;

#[derive(Debug, Deserialize)]
struct CharacterResponse {
    name: String,
    corporation_id: i64,
}

#[derive(Debug, Deserialize)]
struct CorporationResponse {
    name: String,
    alliance_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct AllianceResponse {
    name: String,
}

pub struct AffiliationService {
    db: Arc<crate::DB>,
    esi_client: crate::core::esi::ESIClient,
}

impl AffiliationService {
    pub fn new(
        database: Arc<crate::DB>,
        esi_client: crate::core::esi::ESIClient,
    ) -> AffiliationService {
        AffiliationService {
            db: database,
            esi_client,
        }
    }

    pub async fn update_character_affiliation(&self, id: i64) -> Result<(), Madness> {
        let character: CharacterResponse = self
            .esi_client
            .get_unauthenticated(&format!("/latest/characters/{}", id))
            .await?;
        self.update_corp_affiliation(character.corporation_id)
            .await?;

        if let None = sqlx::query!("SELECT * FROM `character` WHERE id=?", id)
            .fetch_optional(self.db.as_ref())
            .await?
        {
            sqlx::query!(
                "INSERT INTO `character` (id, name, corporation_id) VALUES (?, ?, ?)",
                id,
                character.name,
                character.corporation_id
            )
            .execute(self.db.as_ref())
            .await?;
        } else {
            sqlx::query!(
                "UPDATE `character` SET name=?, corporation_id=? WHERE id=?",
                character.name,
                character.corporation_id,
                id
            )
            .execute(self.db.as_ref())
            .await?;
        }

        Ok(())
    }

    pub async fn update_corp_affiliation(&self, id: i64) -> Result<(), Madness> {
        let corporation = sqlx::query!("SELECT * FROM corporation WHERE id=?", id)
            .fetch_optional(self.db.as_ref())
            .await?;

        let now = chrono::Utc::now().timestamp();

        let mut known: bool = false;
        if let Some(corp) = corporation {
            known = true;

            // If the corp was updated in the last 24h, we don't need to fetch it again
            if corp.updated_at + 60 * 60 * 24 > now {
                return Ok(());
            }
        }

        let esi_res: CorporationResponse = self
            .esi_client
            .get_unauthenticated(&format!("/latest/corporations/{}", id))
            .await?;
        if let Some(alliance_id) = esi_res.alliance_id {
            self.update_alliance(alliance_id).await?;
        }

        if !known {
            sqlx::query!(
                "INSERT INTO corporation (id, name, alliance_id, updated_at) VALUES (?, ?, ?, ?)",
                id,
                esi_res.name,
                esi_res.alliance_id,
                now
            )
            .execute(self.db.as_ref())
            .await?;
        } else {
            sqlx::query!(
                "UPDATE corporation SET name=?, alliance_id=?, updated_at=? WHERE id=?",
                esi_res.name,
                esi_res.alliance_id,
                now,
                id
            )
            .execute(self.db.as_ref())
            .await?;
        }

        Ok(())
    }

    pub async fn update_alliance(&self, id: i64) -> Result<(), Madness> {
        let esi_res: AllianceResponse = self
            .esi_client
            .get_unauthenticated(&format!("/latest/alliances/{}", id))
            .await?;
        if let None = sqlx::query!("SELECT * FROM alliance WHERE id=?", id)
            .fetch_optional(self.db.as_ref())
            .await?
        {
            sqlx::query!(
                "INSERT INTO alliance (`id`, `name`) VALUES (?, ?)",
                id,
                esi_res.name
            )
            .fetch_optional(self.db.as_ref())
            .await?;
        } else {
            sqlx::query!("UPDATE alliance SET name=? WHERE id=?", esi_res.name, id)
                .fetch_optional(self.db.as_ref())
                .await?;
        }

        Ok(())
    }

    /// Bulk-update character corporation_ids via POST /characters/affiliation/.
    /// Ensures corporation (and alliance) rows exist for FK constraints.
    pub async fn update_characters_affiliation_bulk(
        &self,
        character_ids: &[i64],
    ) -> Result<(), Madness> {
        if character_ids.is_empty() {
            return Ok(());
        }

        // ESI allows up to 1000 ids per request; chunk just in case.
        for chunk in character_ids.chunks(1000) {
            let affiliations = self
                .esi_client
                .get_character_affiliations(chunk)
                .await?;

            let mut corp_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
            for aff in &affiliations {
                corp_ids.insert(aff.corporation_id);
            }
            for corp_id in corp_ids {
                self.update_corp_affiliation(corp_id).await?;
            }

            let names = self.esi_client.get_bulk_names(chunk).await.unwrap_or_default();

            for aff in affiliations {
                let resolved_name = names.get(&aff.character_id).cloned();

                if let None = sqlx::query!("SELECT id FROM `character` WHERE id=?", aff.character_id)
                    .fetch_optional(self.db.as_ref())
                    .await?
                {
                    // Only invent a placeholder when inserting a brand-new row with no ESI name.
                    let name = resolved_name.unwrap_or_else(|| {
                        format!("Character {}", aff.character_id)
                    });
                    sqlx::query!(
                        "INSERT INTO `character` (id, name, corporation_id) VALUES (?, ?, ?)",
                        aff.character_id,
                        name,
                        aff.corporation_id
                    )
                    .execute(self.db.as_ref())
                    .await?;
                } else if let Some(name) = resolved_name {
                    sqlx::query!(
                        "UPDATE `character` SET name=?, corporation_id=? WHERE id=?",
                        name,
                        aff.corporation_id,
                        aff.character_id
                    )
                    .execute(self.db.as_ref())
                    .await?;
                } else {
                    // Keep existing name if bulk /universe/names/ omitted this id.
                    sqlx::query!(
                        "UPDATE `character` SET corporation_id=? WHERE id=?",
                        aff.corporation_id,
                        aff.character_id
                    )
                    .execute(self.db.as_ref())
                    .await?;
                }
            }
        }

        Ok(())
    }
}
