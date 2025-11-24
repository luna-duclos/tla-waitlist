use std::collections::{HashMap, HashSet};

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
struct RoleAssignment {
    name: String,
    in_fleet: bool,
    correct_hull: Option<bool>, // None if not in fleet, Some(true) if correct hull, Some(false) if wrong hull
    hull_id: Option<i64>, // Ship type ID if in fleet, None otherwise
}

#[derive(Debug, Serialize)]
struct FleetCompResponse {
    wings: Vec<FleetCompWing>,
    id: i64,
    member: Option<FleetMember>,
    role_assignments: Option<HashMap<String, Vec<RoleAssignment>>>,
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

fn strip_html_tags(text: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {
                if !in_tag {
                    result.push(ch);
                }
            }
        }
    }
    
    result
}

fn extract_names_from_html_tags(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut chars = text.chars().peekable();
    let mut current_name = String::new();
    let mut in_tag = false;
    let mut in_closing_tag = false;
    
    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                // Check if this is a closing tag
                if chars.peek() == Some(&'/') {
                    // Save the name we were collecting before the closing tag
                    if !current_name.trim().is_empty() {
                        names.push(current_name.trim().to_string());
                        current_name.clear();
                    }
                    in_closing_tag = true;
                    chars.next(); // consume the '/'
                } else {
                    // If we were collecting a name, save it (shouldn't happen normally)
                    if !current_name.trim().is_empty() {
                        names.push(current_name.trim().to_string());
                        current_name.clear();
                    }
                    in_tag = true;
                }
            }
            '>' => {
                in_tag = false;
                in_closing_tag = false;
            }
            _ => {
                if in_tag || in_closing_tag {
                    // Skip tag content
                    continue;
                } else {
                    // We're outside tags - this is character name content
                    current_name.push(ch);
                }
            }
        }
    }
    
    // Don't forget the last name (if text ends without a closing tag)
    if !current_name.trim().is_empty() {
        names.push(current_name.trim().to_string());
    }
    
    // Only return if we found HTML tags (indicated by having HTML structure)
    if !names.is_empty() && text.contains('<') {
        return names.into_iter().filter(|s| !s.is_empty()).collect();
    }
    
    Vec::new()
}

fn parse_character_names(text: &str) -> Vec<String> {
    // First, try to extract names from HTML tags (e.g., <tag>Name1</tag> <tag>Name2</tag>)
    let html_names = extract_names_from_html_tags(text);
    if !html_names.is_empty() {
        return html_names;
    }
    
    // If no HTML tags found, strip HTML and parse normally
    let cleaned = strip_html_tags(text).trim().to_string();
    if cleaned.is_empty() {
        return Vec::new();
    }
    
    let delimiters = [',', ';', '|', '/', '\\'];
    
    // Check if any delimiter exists
    let has_delimiter = delimiters.iter().any(|&delim| cleaned.contains(delim));
    
    if has_delimiter {
        // Split by the first delimiter found
        for &delim in &delimiters {
            if cleaned.contains(delim) {
                return cleaned
                    .split(delim)
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    
    // If no delimiter found, try to split by multiple spaces or newlines
    // This handles cases like "Character1  Character2" or names on separate lines
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    
    // If we have multiple words, they might be separate character names
    // But we need to be careful - single character names might have spaces
    // For now, if there are multiple space-separated tokens, treat them as separate names
    if parts.len() > 1 {
        parts.into_iter().map(|s| s.to_string()).collect()
    } else {
        // Single name (might contain spaces, which is fine)
        vec![cleaned]
    }
}

fn extract_names_from_anchor_tags_after_role(motd: &str, role: &str, roles: &[&str]) -> Vec<String> {
    let mut names = Vec::new();
    let role_pattern = format!("{}:", role.to_lowercase());
    let motd_lower = motd.to_lowercase();
    
    // Find the role pattern in the HTML - need to find it as a complete label
    // Look for the pattern followed by whitespace, HTML tag, or end of text
    // This might break btw if there is malformed tags or whatever. also you need the colon. 
    let mut search_positions = Vec::new();
    let mut search_start_pos = 0;
    
    while let Some(role_pos) = motd_lower[search_start_pos..].find(&role_pattern) {
        let actual_pos = search_start_pos + role_pos;
        let after_pattern = actual_pos + role_pattern.len();
        
        // Check if this is a valid role label (followed by whitespace, tag, or end)
        if after_pattern >= motd.len() {
            // End of string - valid
            search_positions.push(actual_pos);
            break;
        } else {
            let next_char = motd.chars().nth(after_pattern);
            if let Some(ch) = next_char {
                // Valid if followed by whitespace, HTML tag start, or closing tag
                if ch.is_whitespace() || ch == '<' || ch == '>' {
                    search_positions.push(actual_pos);
                }
            }
        }
        
        search_start_pos = actual_pos + 1;
    }
    
    // Use the first valid match (should be the correct one)
    if let Some(&role_pos) = search_positions.first() {
        // Find the position after the role label
        let search_start = role_pos + role_pattern.len();
        
        // Find where the next role starts (if any)
        let mut search_end = motd.len();
        for other_role in roles {
            if other_role.to_lowercase() != role.to_lowercase() {
                let other_pattern = format!("{}:", other_role.to_lowercase());
                if let Some(other_pos) = motd_lower[search_start..].find(&other_pattern) {
                    let actual_pos = search_start + other_pos;
                    if actual_pos < search_end {
                        search_end = actual_pos;
                    }
                }
            }
        }
        
        // Extract all <a> tag contents in this section
        let section = &motd[search_start..search_end];
        let mut chars = section.chars().peekable();
        let mut in_anchor_tag = false;
        let mut current_name = String::new();
        let mut tag_depth = 0; // Track depth of nested tags inside anchor
        
        while let Some(ch) = chars.next() {
            if ch == '<' {
                // Check if this is an anchor tag
                let mut is_closing = false;
                let mut tag_name = String::new();
                
                // Check for closing tag
                if chars.peek() == Some(&'/') {
                    is_closing = true;
                    chars.next(); // consume '/'
                }
                
                // Read tag name until space or >
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == ' ' || next_ch == '>' {
                        break;
                    }
                    if let Some(c) = chars.next() {
                        tag_name.push(c);
                    }
                }
                
                let tag_lower = tag_name.to_lowercase();
                
                // Skip rest of tag until '>'
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '>' {
                        chars.next(); // consume '>'
                        break;
                    }
                    chars.next(); // skip attribute content
                }
                
                if tag_lower == "a" {
                    if is_closing {
                        // Closing </a> tag - save the name we collected
                        if !current_name.trim().is_empty() {
                            names.push(current_name.trim().to_string());
                            current_name.clear();
                        }
                        in_anchor_tag = false;
                        tag_depth = 0;
                    } else {
                        // Opening <a> tag
                        in_anchor_tag = true;
                        tag_depth = 1;
                    }
                } else if in_anchor_tag {
                    // We're inside an anchor tag - handle nested tags
                    if is_closing {
                        tag_depth -= 1;
                        // If we've closed all nested tags, we're back to anchor content
                        if tag_depth <= 0 {
                            tag_depth = 1; // Back to anchor tag level
                        }
                    } else {
                        tag_depth += 1;
                    }
                }
            } else if in_anchor_tag && tag_depth == 1 {
                // We're inside an anchor tag and not in a nested tag - collect the character name
                current_name.push(ch);
            }
        }
        
        // Don't forget the last name if section ends while in an anchor tag
        if !current_name.trim().is_empty() {
            names.push(current_name.trim().to_string());
        }
    }
    
    names.into_iter().filter(|s| !s.is_empty()).collect()
}

fn parse_motd_roles(motd: &str) -> HashMap<String, Vec<String>> {
    let mut role_assignments: HashMap<String, Vec<String>> = HashMap::new();
    let roles = vec!["DDD", "LR", "PS", "MS", "MTAC"];
    
    // First, try to extract names from <a> tags (HTML format)
    for role in &roles {
        let names = extract_names_from_anchor_tags_after_role(motd, role, &roles);
        if !names.is_empty() {
            role_assignments.insert(role.to_string(), names);
            continue; // Skip the text-based parsing for this role
        }
    }
    
    // If we found roles via HTML, return early
    if !role_assignments.is_empty() {
        return role_assignments;
    }
    
    // Fall back to text-based parsing (strip HTML first)
    let motd_clean = strip_html_tags(motd);
    
    // Convert MOTD to lowercase for case-insensitive matching
    let motd_lower = motd_clean.to_lowercase();
    
    for role in &roles {
        let role_lower = role.to_lowercase();
        let mut found_assignments = Vec::new();
        
        // Only match patterns with colon: "ROLE:"
        let pattern = format!("{}:", role_lower);
        
        if let Some(pos) = motd_lower.find(&pattern) {
            let start = pos + pattern.len();
            let remaining = &motd_clean[start..];
            
            // Extract characters until newline, comma, or another role pattern
            let mut char_end = remaining.len();
            for (i, ch) in remaining.char_indices() {
                if ch == '\n' || ch == '\r' {
                    char_end = i;
                    break;
                }
                // Check if we hit another role pattern with colon
                for other_role in &roles {
                    let other_role_lower = other_role.to_lowercase();
                    let other_pattern = format!("{}:", other_role_lower);
                    if i < remaining.len() && remaining[i..].to_lowercase().starts_with(&other_pattern) {
                        char_end = i;
                        break;
                    }
                }
                if char_end != remaining.len() {
                    break;
                }
            }
            
            let char_text = remaining[..char_end].trim();
            if !char_text.is_empty() {
                // Parse character names using flexible delimiter handling
                let characters = parse_character_names(char_text);
                
                for char_name in characters {
                    let char_name_clean = char_name.trim().to_string();
                    if !char_name_clean.is_empty() && !found_assignments.contains(&char_name_clean) {
                        found_assignments.push(char_name_clean);
                    }
                }
            }
        }
        
        // Also try to find role on its own line followed by character names
        let lines: Vec<&str> = motd_clean.split('\n').collect();
        for (i, line) in lines.iter().enumerate() {
            let line_lower_str = line.to_lowercase();
            let line_lower = line_lower_str.trim();
            // Only match if line starts with "ROLE:"
            if line_lower.starts_with(&format!("{}:", role_lower)) {
                // Extract characters from the same line after the colon
                let colon_pos = line_lower.find(':').unwrap_or(0);
                let after_colon = &line[colon_pos + 1..].trim();
                if !after_colon.is_empty() {
                    let characters = parse_character_names(after_colon);
                    for char_name in characters {
                        let char_name_clean = char_name.trim().to_string();
                        if !char_name_clean.is_empty() && !found_assignments.contains(&char_name_clean) {
                            found_assignments.push(char_name_clean);
                        }
                    }
                }
                
                // Check next lines for character names (if line only had the role)
                let mut j = i + 1;
                while j < lines.len() && j < i + 5 {
                    let next_line = lines[j].trim();
                    if !next_line.is_empty() {
                        let next_line_lower = next_line.to_lowercase();
                        // Check if this line contains another role with colon
                        let mut is_role_line = false;
                        for other_role in &roles {
                            let other_role_lower = other_role.to_lowercase();
                            if next_line_lower.starts_with(&format!("{}:", other_role_lower)) {
                                is_role_line = true;
                                break;
                            }
                        }
                        if !is_role_line {
                            let characters = parse_character_names(next_line);
                            for char_name in characters {
                                let char_name_clean = char_name.trim().to_string();
                                if !char_name_clean.is_empty() && !found_assignments.contains(&char_name_clean) {
                                    found_assignments.push(char_name_clean);
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    j += 1;
                }
            }
        }
        
        if !found_assignments.is_empty() {
            role_assignments.insert(role.to_string(), found_assignments);
        }
    }
    
    role_assignments
}

// Function to get allowed hulls for a role
fn get_allowed_hulls_for_role(role: &str) -> Vec<&'static str> {
    match role.to_uppercase().as_str() {
        "DDD" => vec!["vindicator"],
        "MS" => vec!["damnation", "eos", "claymore"], 
        "LR" => vec!["kronos"],
        "MTAC" => vec!["paladin", "vargur", "occator", "mastodon", "bustard", "impel"],
        "PS" => vec!["paladin", "vargur"],
        _ => vec![],
    }
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
    
    // Create a set of fleet member names (case-insensitive) for checking MOTD assignments
    let fleet_member_names: std::collections::HashSet<String> = characters
        .values()
        .map(|char| char.name.to_lowercase())
        .collect();
    
    // Create a mapping of character names to hull names and IDs (case-insensitive)
    let mut character_to_hull: HashMap<String, String> = HashMap::new();
    let mut character_to_hull_id: HashMap<String, i64> = HashMap::new();
    for member in &members {
        if let Some(character) = characters.get(&member.character_id) {
            let hull_name = TypeDB::name_of(member.ship_type_id).unwrap();
            character_to_hull.insert(character.name.to_lowercase(), hull_name.to_lowercase());
            character_to_hull_id.insert(character.name.to_lowercase(), member.ship_type_id as i64);
        }
    }
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

    // Fetch fleet MOTD and parse role assignments
    let role_assignments = match crate::core::esi::fleet_info::get(&app.esi_client, fleet_id, fleet.boss_id).await {
        Ok(fleet_info) => {
            if fleet_info.motd.is_empty() {
                None
            } else {
                let parsed = parse_motd_roles(&fleet_info.motd);
                if parsed.is_empty() {
                    None
                } else {
                    // Convert parsed names to RoleAssignment structs, checking fleet membership and hull
                    let mut role_assignments_with_status: HashMap<String, Vec<RoleAssignment>> = HashMap::new();
                    for (role, names) in parsed {
                        let allowed_hulls: Vec<String> = get_allowed_hulls_for_role(&role)
                            .into_iter()
                            .map(|s| s.to_lowercase())
                            .collect();
                        
                        let assignments: Vec<RoleAssignment> = names
                            .into_iter()
                            .map(|name| {
                                let name_lower = name.to_lowercase();
                                let in_fleet = fleet_member_names.contains(&name_lower);
                                
                                let (correct_hull, hull_id) = if in_fleet {
                                    // Check if the character's hull matches the allowed hulls for this role
                                    if let Some(hull_name) = character_to_hull.get(&name_lower) {
                                        let is_correct = allowed_hulls.contains(hull_name);
                                        let ship_id = character_to_hull_id.get(&name_lower).copied();
                                        (Some(is_correct), ship_id)
                                    } else {
                                        (Some(false), None) // In fleet but hull not found (shouldn't happen)
                                    }
                                } else {
                                    (None, None) // Not in fleet, so hull check doesn't apply
                                };
                                
                                RoleAssignment {
                                    name,
                                    in_fleet,
                                    correct_hull,
                                    hull_id,
                                }
                            })
                            .collect();
                        role_assignments_with_status.insert(role, assignments);
                    }
                    Some(role_assignments_with_status)
                }
            }
        }
        Err(_) => None, // Gracefully handle errors - if MOTD fetch fails, return None
    };

    Ok(Json(FleetCompResponse {
        wings,
        id: fleet_id,
        member: fleet_commander,
        role_assignments,
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

    let categories = crate::data::categories::categories();
    let category_lookup: HashMap<_, _> = categories
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

    let categories = crate::data::categories::categories();
    for category in categories {
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
