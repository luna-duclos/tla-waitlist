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

// Extract href from anchor tag for a character name in the MOTD
fn extract_anchor_href(motd: &str, character_name: &str) -> Option<String> {
    let name_lower = character_name.to_lowercase();
    let motd_lower = motd.to_lowercase();
    
    // Find the character name in the MOTD (case-insensitive)
    if let Some(name_pos) = motd_lower.find(&name_lower) {
        // Look backwards from the name to find the opening <a> tag
        let before_name = &motd[..name_pos];
        if let Some(anchor_start) = before_name.rfind("<a") {
            let anchor_section = &motd[anchor_start..];
            // Extract href attribute
            if let Some(href_start) = anchor_section.find("href=\"") {
                let href_content_start = anchor_start + href_start + 6; // Skip "href=\""
                if let Some(href_end) = motd[href_content_start..].find('"') {
                    return Some(motd[href_content_start..href_content_start + href_end].to_string());
                }
            }
        }
    }
    None
}

// Remove characters not in fleet from all role sections - surgically, preserving HTML structure
fn remove_characters_not_in_fleet_surgical(
    motd: &str,
    fleet_member_names: &HashSet<String>,
) -> String {
    let roles = vec!["DDD", "MS", "LR", "MTAC", "PS"];
    let mut result = motd.to_string();
    
    // Process roles in reverse order to avoid position shifts
    for role in roles.iter().rev() {
        let role_pattern = format!("{}:", role.to_lowercase());
        let mut result_lower = result.to_lowercase();
        
        // Keep processing until no more characters to remove
        loop {
            let mut found_any = false;
            
            // Use regex to find role pattern with word boundaries to avoid false matches
            use regex::Regex;
            let role_regex = Regex::new(&format!(r"(?i)\b{}:\s*", regex::escape(role))).unwrap();
            
            if let Some(role_match) = role_regex.find(&result_lower) {
                let role_start = role_match.end();
                let mut role_end = result.len();
                
                // Find where this role section ends - look for next role pattern with word boundaries
                for other_role in &roles {
                    if other_role.to_lowercase() != role.to_lowercase() {
                        let other_pattern_regex = Regex::new(&format!(r"(?i)\b{}:\s*", regex::escape(other_role))).unwrap();
                        if let Some(other_match) = other_pattern_regex.find(&result_lower[role_start..]) {
                            let actual_pos = role_start + other_match.start();
                            if actual_pos < role_end {
                                role_end = actual_pos;
                            }
                        }
                    }
                }
                
                // Get all character names in this role section
                let parsed_names = extract_names_from_anchor_tags_after_role(&result, role, &roles);
                
                // For each character not in fleet, surgically remove their anchor tag
                for name in &parsed_names {
                    let name_trimmed = name.trim();
                    let name_lower = name_trimmed.to_lowercase();
                    
                    if !fleet_member_names.contains(&name_lower) {
                        let section = &result[role_start..role_end];
                        
                        // Find all anchor tags in the section and match by exact name content
                        // Search only within the role section, not the entire MOTD
                        let mut anchor_start = 0;
                        let mut anchor_count = 0;
                        while let Some(anchor_pos) = section[anchor_start..].find("<a") {
                            anchor_count += 1;
                            // anchor_pos is relative to anchor_start in section
                            // section starts at role_start, so absolute position is:
                            let anchor_abs_pos = role_start + anchor_start + anchor_pos;
                            
                            // Make sure we don't go beyond role_end
                            if anchor_abs_pos >= role_end {
                                break;
                            }
                            
                            // Limit the search to only within the role section
                            let anchor_section = &result[anchor_abs_pos..role_end];
                            
                            if let Some(anchor_tag_end) = anchor_section.find('>') {
                                let anchor_tag_end_absolute = anchor_abs_pos + anchor_tag_end + 1;
                                
                                // Find the closing </a> tag - search only within role section
                                // anchor_tag_end is relative to anchor_section, which starts at anchor_abs_pos
                                // So we need to search from anchor_tag_end position in anchor_section
                                let search_start = anchor_tag_end;
                                if let Some(anchor_close) = anchor_section[search_start..].find("</a>") {
                                    // anchor_close is relative to search_start, which is relative to anchor_section start (anchor_abs_pos)
                                    let anchor_close_absolute = anchor_abs_pos + search_start + anchor_close;
                                    
                                    // Make sure the entire anchor tag is within the role section
                                    // anchor_close_absolute points to the '<' of '</a>', so +4 gives us the position after '</a>'
                                    let anchor_end_absolute = anchor_close_absolute + 4; // +4 for "</a>"
                                    
                                    // Verify we're not going beyond the role section
                                    if anchor_end_absolute > role_end {
                                        // This anchor extends beyond the role section, skip it
                                        anchor_start += anchor_pos + 2;
                                        continue;
                                    }
                                    
                                    if anchor_end_absolute > role_end {
                                        // This anchor extends beyond the role section, skip it
                                        anchor_start += anchor_pos + 2;
                                        continue;
                                    }
                                    
                                    // Extract the name from this anchor tag
                                    let anchor_content = &result[anchor_tag_end_absolute..anchor_close_absolute];
                                    
                                    // Strip HTML tags from the anchor content for comparison
                                    let mut stripped_name = String::new();
                                    let mut in_tag = false;
                                    for ch in anchor_content.chars() {
                                        if ch == '<' {
                                            in_tag = true;
                                        } else if ch == '>' {
                                            in_tag = false;
                                        } else if !in_tag {
                                            stripped_name.push(ch);
                                        }
                                    }
                                    let stripped_name_lower = stripped_name.trim().to_lowercase();
                                    
                                    // Match if the stripped name matches (case-insensitive)
                                    if stripped_name_lower == name_lower {
                                        // Found the matching anchor tag - remove it
                                        // anchor_close_absolute points to the '<' of '</a>'
                                        // So anchor_close_absolute + 4 points to the position AFTER '</a>'
                                        let anchor_end_absolute = anchor_close_absolute + 4; // +4 for "</a>"
                                        
                                        // Make sure we don't go beyond the string length
                                        let actual_end = anchor_end_absolute.min(result.len());
                                        let after_anchor = &result[actual_end..];
                                        
                                        let mut new_result = String::new();
                                        new_result.push_str(&result[..anchor_abs_pos]);
                                        
                                        // Check what comes after the anchor - preserve delimiters appropriately
                                        let trimmed_after = after_anchor.trim_start();
                                        
                                        // Skip leading comma and/or space after the anchor
                                        let mut skip = after_anchor.len() - trimmed_after.len();
                                        if trimmed_after.starts_with(',') {
                                            skip += 1;
                                            if trimmed_after.len() > 1 && trimmed_after.chars().nth(1) == Some(' ') {
                                                skip += 1;
                                            }
                                        }
                                        
                                        // Use actual_end instead of anchor_end_absolute to avoid going beyond string length
                                        new_result.push_str(&result[actual_end + skip..]);
                                        
                                        result = new_result;
                                        result_lower = result.to_lowercase();
                                        found_any = true;
                                        break; // Break inner loop, will continue outer loop
                                    }
                                }
                            }
                            anchor_start += anchor_pos + 2;
                        }
                        
                        if found_any {
                            break; // Break out of name loop to restart role processing
                        }
                    }
                }
            }
            
            if !found_any {
                break; // No more characters to remove for this role
            }
        }
    }
    
    result
}

// Add or replace a character in a specific role section of the MOTD
// This function surgically replaces only the character name and ID in the anchor tag
fn add_character_to_motd_role(
    original_motd: &str,
    role: &str,
    character_name: &str,
    character_id: i64,
    character_name_to_id: &HashMap<String, i64>,
    fleet_member_names: &HashSet<String>,
) -> String {
    let roles = vec!["DDD", "MS", "LR", "MTAC", "PS"];
    let role_pattern = format!("{}:", role.to_lowercase());
    let motd_lower = original_motd.to_lowercase();
    let character_name_lower = character_name.to_lowercase();
    
    // Find the role section
    if let Some(role_pos) = motd_lower.find(&role_pattern) {
        // Find where this role section ends (next role or end of MOTD)
        let role_start = role_pos + role_pattern.len();
        let mut role_end = original_motd.len();
        
        for other_role in &roles {
            if other_role.to_lowercase() != role.to_lowercase() {
                let other_pattern = format!("{}:", other_role.to_lowercase());
                if let Some(other_pos) = motd_lower[role_start..].find(&other_pattern) {
                    let actual_pos = role_start + other_pos;
                    if actual_pos < role_end {
                        role_end = actual_pos;
                    }
                }
            }
        }
        
        // Extract the role section
        let role_section = &original_motd[role_start..role_end];
        let section_lower = role_section.to_lowercase();
        
        let has_anchor_tags = role_section.contains("<a");
        
        let mut character_found = false;
        let mut existing_name_in_motd = None;
        let mut existing_characters_in_fleet = Vec::new(); // Track characters in fleet in this role
        let mut anchor_start = 0;
        while let Some(anchor_pos) = role_section[anchor_start..].find("<a") {
            let anchor_abs_pos = role_start + anchor_start + anchor_pos;
            let anchor_section = &original_motd[anchor_abs_pos..];
            if let Some(anchor_close) = anchor_section.find('>') {
                let name_start = anchor_abs_pos + anchor_close + 1;
                if let Some(name_end) = original_motd[name_start..].find('<') {
                    let name_in_motd = &original_motd[name_start..name_start + name_end];
                    let name_in_motd_lower = name_in_motd.to_lowercase();
                    
                    // Check if this is the character we're adding
                    if name_in_motd_lower == character_name_lower {
                        character_found = true;
                        existing_name_in_motd = Some(name_in_motd);
                    }
                    
                    // Track if this character is in fleet
                    if fleet_member_names.contains(&name_in_motd_lower) {
                        existing_characters_in_fleet.push(anchor_abs_pos);
                    }
                }
            }
            anchor_start += anchor_pos + 2;
        }
        
        // If character is already in the role, update them (replace)
        if has_anchor_tags && character_found {
            let name_to_find = existing_name_in_motd.unwrap();
            if let Some(name_pos_in_section) = section_lower.find(&name_to_find.to_lowercase()) {
            let name_pos_approx = role_start + name_pos_in_section;
            let before_name = &original_motd[role_start..name_pos_approx];
            if let Some(anchor_start_rel) = before_name.rfind("<a") {
                let anchor_start_absolute = role_start + anchor_start_rel;
                
                let anchor_section = &original_motd[anchor_start_absolute..];
                if let Some(anchor_tag_end) = anchor_section.find('>') {
                    let anchor_tag_end_absolute = anchor_start_absolute + anchor_tag_end + 1;
                    let anchor_content = &original_motd[anchor_tag_end_absolute..];
                    let name_start = if let Some(name_pos_in_anchor) = anchor_content.find(character_name) {
                        anchor_tag_end_absolute + name_pos_in_anchor
                    } else {
                        name_pos_approx
                    };
                    let name_end = name_start + character_name.len();
                    
                    if let Some(href_start) = anchor_section.find("href=\"") {
                        let href_content_start = anchor_start_absolute + href_start + 6;
                        
                        if let Some(href_end) = original_motd[href_content_start..].find('"') {
                            let href_end_absolute = href_content_start + href_end;
                            let href_content = &original_motd[href_content_start..href_end_absolute];
                            
                            if let Some(id_start_in_href) = href_content.find("//") {
                                let id_start_absolute = href_content_start + id_start_in_href + 2;
                                let id_end_absolute = href_end_absolute;
                                
                                let anchor_tag_close = anchor_start_absolute + anchor_section.find('>').unwrap() + 1;
                                let anchor_content_after_gt = &original_motd[anchor_tag_close..];
                                let name_start_exact = if let Some(name_pos) = anchor_content_after_gt.find(character_name) {
                                    anchor_tag_close + name_pos
                                } else {
                                    name_start
                                };
                                let name_end_exact = name_start_exact + character_name.len();
                                
                                let mut result = String::new();
                                result.push_str(&original_motd[..id_start_absolute]);
                                result.push_str(&character_id.to_string());
                                result.push_str(&original_motd[id_end_absolute..anchor_tag_close]);
                                result.push_str(&original_motd[anchor_tag_close..name_start_exact]);
                                result.push_str(character_name);
                                result.push_str(&original_motd[name_end_exact..]);
                                
                                return result;
                            }
                        }
                    }
                }
            }
            }
        }
        
        // If character is NOT in the role, but there are existing characters in fleet, add after the last one
        if has_anchor_tags && !character_found && !existing_characters_in_fleet.is_empty() {
            // Find the last character in fleet (the rightmost one)
            let last_char_pos = *existing_characters_in_fleet.iter().max().unwrap();
            
            // Find the end of that anchor tag
            let anchor_section = &original_motd[last_char_pos..];
            if let Some(anchor_close) = anchor_section.find("</a>") {
                let insert_pos = last_char_pos + anchor_close + 4; // +4 for "</a>"
                
                // Check what comes after the last anchor
                let after_last_anchor = &original_motd[insert_pos..role_end.min(original_motd.len())];
                let trimmed_after = after_last_anchor.trim_start();
                
                // Determine delimiter - use space if there's no comma, otherwise use ", "
                let delimiter = if trimmed_after.starts_with(',') {
                    ", "
                } else if trimmed_after.is_empty() || trimmed_after.starts_with('<') {
                    " "
                } else {
                    " "
                };
                
                // Get showinfo type from existing anchors
                let showinfo_type = if let Some(existing_anchor) = role_section.find("<a") {
                    let anchor_part = &role_section[existing_anchor..];
                    if let Some(href_start) = anchor_part.find("href=\"") {
                        let href_content = &anchor_part[href_start + 6..];
                        if let Some(href_end) = href_content.find('"') {
                            let href = &href_content[..href_end];
                            if let Some(type_start) = href.find("showinfo:") {
                                let after_showinfo = &href[type_start + 9..];
                                if let Some(type_end) = after_showinfo.find("//") {
                                    &after_showinfo[..type_end]
                                } else {
                                    "1377"
                                }
                            } else {
                                "1377"
                            }
                        } else {
                            "1377"
                        }
                    } else {
                        "1377"
                    }
                } else {
                    "1377"
                };
                
                let mut result = String::new();
                result.push_str(&original_motd[..insert_pos]);
                result.push_str(delimiter);
                let href = format!("showinfo:{}//{}", showinfo_type, character_id);
                result.push_str(&format!("<a href=\"{}\">{}</a>", href, character_name));
                result.push_str(&original_motd[insert_pos..]);
                
                return result;
            }
        }
        
        if !has_anchor_tags {
            let role_content = &original_motd[role_start..role_end];
            let trimmed_content = role_content.trim();
            let has_content = !trimmed_content.is_empty();
            
            let mut content_start = role_start;
            for (idx, ch) in role_content.char_indices() {
                if !ch.is_whitespace() {
                    content_start = role_start + idx;
                    break;
                }
            }
            
            let showinfo_type = "1377";
            let mut result = String::new();
            result.push_str(&original_motd[..role_start]);
            result.push(' ');
            
            let href = format!("showinfo:{}//{}", showinfo_type, character_id);
            result.push_str(&format!("<a href=\"{}\">{}</a>", href, character_name));
            result.push_str("<br>");
            result.push_str(&original_motd[role_end..]);
            
            return result;
        }
        
        // If there's only one anchor tag and the character is not found, check if that character is in fleet
        let anchor_count = role_section.matches("<a").count();
        
        if anchor_count == 1 && !character_found {
            if let Some(anchor_start_rel) = role_section.find("<a") {
                let anchor_start_absolute = role_start + anchor_start_rel;
                let anchor_section = &original_motd[anchor_start_absolute..];
                if let Some(anchor_tag_end) = anchor_section.find('>') {
                    let anchor_tag_end_absolute = anchor_start_absolute + anchor_tag_end + 1;
                    if let Some(anchor_close) = anchor_section.find("</a>") {
                        let anchor_close_absolute = anchor_start_absolute + anchor_close;
                        
                        // Extract the existing character's name
                        let existing_name = &original_motd[anchor_tag_end_absolute..anchor_close_absolute];
                        // Strip HTML tags from the name
                        let mut existing_name_clean = String::new();
                        let mut in_tag = false;
                        for ch in existing_name.chars() {
                            if ch == '<' {
                                in_tag = true;
                            } else if ch == '>' {
                                in_tag = false;
                            } else if !in_tag {
                                existing_name_clean.push(ch);
                            }
                        }
                        let existing_name_lower = existing_name_clean.trim().to_lowercase();
                        
                        // If the existing character is in fleet, add the new character after them
                        if fleet_member_names.contains(&existing_name_lower) {
                            let insert_pos = anchor_close_absolute + 4; // +4 for "</a>"
                            
                            let delimiter = " ";
                            
                            // Get showinfo type from existing anchor
                            let showinfo_type = if let Some(href_start) = anchor_section.find("href=\"") {
                                let href_content = &anchor_section[href_start + 6..];
                                if let Some(href_end) = href_content.find('"') {
                                    let href = &href_content[..href_end];
                                    if let Some(type_start) = href.find("showinfo:") {
                                        let after_showinfo = &href[type_start + 9..];
                                        if let Some(type_end) = after_showinfo.find("//") {
                                            &after_showinfo[..type_end]
                                        } else {
                                            "1377"
                                        }
                                    } else {
                                        "1377"
                                    }
                                } else {
                                    "1377"
                                }
                            } else {
                                "1377"
                            };
                            
                            let mut result = String::new();
                            result.push_str(&original_motd[..insert_pos]);
                            result.push_str(delimiter);
                            let href = format!("showinfo:{}//{}", showinfo_type, character_id);
                            result.push_str(&format!("<a href=\"{}\">{}</a>", href, character_name));
                            result.push_str(&original_motd[insert_pos..]);
                            
                            return result;
                        } else {
                            // Existing character is not in fleet, replace them
                            if let Some(href_start) = anchor_section.find("href=\"") {
                                let href_content_start = anchor_start_absolute + href_start + 6;
                                if let Some(href_end) = original_motd[href_content_start..].find('"') {
                                    let href_end_absolute = href_content_start + href_end;
                                    let href_content = &original_motd[href_content_start..href_end_absolute];
                                    if let Some(id_start_in_href) = href_content.find("//") {
                                        let id_start_absolute = href_content_start + id_start_in_href + 2;
                                        let id_end_absolute = href_end_absolute;
                                        let name_start = anchor_tag_end_absolute;
                                        let name_end = anchor_close_absolute;
                                        
                                        let mut result = String::new();
                                        result.push_str(&original_motd[..id_start_absolute]);
                                        result.push_str(&character_id.to_string());
                                        result.push_str(&original_motd[id_end_absolute..name_start]);
                                        result.push_str(character_name);
                                        result.push_str(&original_motd[name_end..]);
                                        return result;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Multiple characters - add to the end
        // Find the last </a> tag in the section
        if let Some(last_anchor_end) = role_section.rfind("</a>") {
            let insert_pos = role_start + last_anchor_end + 4;
            
            let after_last_anchor = &original_motd[insert_pos..role_end];
            let delimiter = if after_last_anchor.trim_start().starts_with(',') {
                ", "
            } else if after_last_anchor.trim_start().is_empty() || after_last_anchor.trim_start().starts_with('<') {
                ""
            } else {
                ", "
            };
            let showinfo_type = if let Some(existing_anchor) = role_section.find("<a") {
                let anchor_part = &role_section[existing_anchor..];
                if let Some(href_start) = anchor_part.find("href=\"") {
                    let href_content = &anchor_part[href_start + 6..];
                    if let Some(href_end) = href_content.find('"') {
                        let href = &href_content[..href_end];
                        if let Some(type_start) = href.find("showinfo:") {
                            let after_showinfo = &href[type_start + 9..];
                            if let Some(type_end) = after_showinfo.find("//") {
                                &after_showinfo[..type_end]
                            } else {
                                "1377"
                            }
                        } else {
                            "1377"
                        }
                    } else {
                        "1377"
                    }
                } else {
                    "1377"
                }
            } else {
                "1377"
            };
            
            let mut result = String::new();
            result.push_str(&original_motd[..insert_pos]);
            result.push_str(delimiter);
            let href = format!("showinfo:{}//{}", showinfo_type, character_id);
            result.push_str(&format!("<a href=\"{}\">{}</a>", href, character_name));
            result.push_str(&original_motd[insert_pos..]);
            
            return result;
        }
    }
    
    // Role not found, append it at the end
    let mut result = original_motd.to_string();
    if !result.trim().is_empty() && !result.trim_end().ends_with('\n') {
        result.push('\n');
    }
    result.push_str(&format!("{}: ", role.to_uppercase()));
    let href = format!("showinfo:1377//{}", character_id);
    result.push_str(&format!("<a href=\"{}\">{}</a>", href, character_name));
    result.push('\n');
    
    result
}

// Reconstruct MOTD by removing characters not in fleet and updating the specific role
fn reconstruct_motd(
    original_motd: &str,
    role_assignments: &HashMap<String, Vec<String>>,
    character_name_to_id: &HashMap<String, i64>,
    fleet_member_names: &HashSet<String>,
    updated_role: Option<&str>,
    updated_character: Option<&str>,
) -> String {
    let roles = vec!["DDD", "MS", "LR", "MTAC", "PS"];
    let mut result = original_motd.to_string();
    
    // If we're updating a specific role, use the surgical approach
    if let (Some(role), Some(character)) = (updated_role, updated_character) {
        if let Some(&char_id) = character_name_to_id.get(&character.to_lowercase()) {
            result = add_character_to_motd_role(
                &result,
                role,
                character,
                char_id,
                character_name_to_id,
                fleet_member_names,
            );
        }
    }
    
    // Remove characters not in fleet from all role sections
    for role in &roles {
        let role_pattern = format!("{}:", role.to_lowercase());
        let result_lower = result.to_lowercase();
        
        if let Some(role_pos) = result_lower.find(&role_pattern) {
            let role_start = role_pos + role_pattern.len();
            let mut role_end = result.len();
            
            // Find where this role section ends
            for other_role in &roles {
                if other_role.to_lowercase() != role.to_lowercase() {
                    let other_pattern = format!("{}:", other_role.to_lowercase());
                    if let Some(other_pos) = result_lower[role_start..].find(&other_pattern) {
                        let actual_pos = role_start + other_pos;
                        if actual_pos < role_end {
                            role_end = actual_pos;
                        }
                    }
                }
            }
            
            let role_section = &result[role_start..role_end];
            let mut new_section = String::new();
            let mut in_anchor = false;
            let mut current_anchor = String::new();
            let mut current_name = String::new();
            let mut tag_depth = 0;
            
            // Parse through the section and keep only characters in fleet
            let mut chars = role_section.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '<' {
                    let mut is_closing = false;
                    let mut tag_name = String::new();
                    
                    if chars.peek() == Some(&'/') {
                        is_closing = true;
                        chars.next();
                    }
                    
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch == ' ' || next_ch == '>' {
                            break;
                        }
                        if let Some(c) = chars.next() {
                            tag_name.push(c);
                        }
                    }
                    
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch == '>' {
                            chars.next();
                            break;
                        }
                        chars.next();
                    }
                    
                    let tag_lower = tag_name.to_lowercase();
                    
                    if tag_lower == "a" {
                        if is_closing {
                            // Closing </a> tag - check if we should keep this character
                            if !current_name.trim().is_empty() {
                                if fleet_member_names.contains(&current_name.trim().to_lowercase()) {
                                    new_section.push_str(&current_anchor);
                                    new_section.push_str("</a>");
                                }
                            }
                            current_anchor.clear();
                            current_name.clear();
                            in_anchor = false;
                            tag_depth = 0;
                        } else {
                            // Opening <a> tag
                            in_anchor = true;
                            tag_depth = 1;
                            current_anchor.push(ch);
                            current_anchor.push(if is_closing { '/' } else { ' ' });
                            // Reconstruct the opening tag
                            let mut tag_rebuild = String::from("<");
                            if is_closing {
                                tag_rebuild.push('/');
                            }
                            tag_rebuild.push_str(&tag_name);
                            // Skip to '>'
                            let mut attr_part = String::new();
                            while let Some(&next_ch) = chars.peek() {
                                if next_ch == '>' {
                                    chars.next();
                                    attr_part.push(next_ch);
                                    break;
                                }
                                if let Some(c) = chars.next() {
                                    attr_part.push(c);
                                }
                            }
                            current_anchor = format!("<{}", tag_name);
                            current_anchor.push_str(&attr_part);
                        }
                    } else if in_anchor {
                        current_anchor.push(ch);
                        if is_closing {
                            tag_depth -= 1;
                        } else {
                            tag_depth += 1;
                        }
                    } else {
                        new_section.push(ch);
                    }
                } else if in_anchor && tag_depth == 1 {
                    current_name.push(ch);
                    current_anchor.push(ch);
                } else {
                    if in_anchor {
                        current_anchor.push(ch);
                    } else {
                        new_section.push(ch);
                    }
                }
            }
            
            // Replace the role section in result
            let mut new_result = String::new();
            new_result.push_str(&result[..role_start]);
            new_result.push_str(&new_section);
            new_result.push_str(&result[role_end..]);
            result = new_result;
        }
    }
    
    result
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

#[derive(Debug, Deserialize)]
struct UpdateMotdRequest {
    character_id: i64,
    role: String,
    character_name: String,
}

#[derive(Debug, Serialize)]
struct UpdateMotdResponse {
    success: bool,
    message: String,
    is_duplicate: bool,
    has_incorrect_hull: bool,
}

#[post("/api/fleet/update-motd", data = "<input>")]
async fn update_motd(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
    input: Json<UpdateMotdRequest>,
) -> Result<Json<UpdateMotdResponse>, Madness> {
    account.require_access("fleet-configure")?;
    authorize_character(app.get_db(), &account, input.character_id, None).await?;
    
    let fleet_id = get_current_fleet_id(app, input.character_id).await?;
    let fleet = match sqlx::query!("SELECT boss_id FROM fleet WHERE id = ?", fleet_id)
        .fetch_optional(app.get_db())
        .await?
    {
        Some(fleet) => fleet,
        None => return Err(Madness::NotFound("Fleet not configured")),
    };
    
    // Fetch current MOTD
    let fleet_info = crate::core::esi::fleet_info::get(&app.esi_client, fleet_id, fleet.boss_id).await?;
    let current_motd = fleet_info.motd;
    
    // Parse current role assignments
    let mut role_assignments = parse_motd_roles(&current_motd);
    
    // Get current fleet members
    let members = crate::core::esi::fleet_members::get(&app.esi_client, fleet_id, fleet.boss_id).await?;
    let character_ids: Vec<_> = members.iter().map(|member| member.character_id).collect();
    let characters = crate::data::character::lookup(app.get_db(), &character_ids).await?;
    
    // Create mappings
    let fleet_member_names: HashSet<String> = characters
        .values()
        .map(|char| char.name.to_lowercase())
        .collect();
    
    let mut character_name_to_id: HashMap<String, i64> = HashMap::new();
    for (char_id, char) in &characters {
        character_name_to_id.insert(char.name.to_lowercase(), *char_id);
    }
    
    let mut character_to_hull: HashMap<String, String> = HashMap::new();
    for member in &members {
        if let Some(character) = characters.get(&member.character_id) {
            let hull_name = TypeDB::name_of(member.ship_type_id).unwrap();
            character_to_hull.insert(character.name.to_lowercase(), hull_name.to_lowercase());
        }
    }
    
    // Check for duplicate
    let role_upper = input.role.to_uppercase();
    let character_name_lower = input.character_name.to_lowercase();
    let is_duplicate = role_assignments
        .get(&role_upper)
        .map(|names| names.iter().any(|n| n.to_lowercase() == character_name_lower))
        .unwrap_or(false);
    
    // Check hull
    let has_incorrect_hull = if let Some(hull_name) = character_to_hull.get(&character_name_lower) {
        let allowed_hulls: Vec<String> = get_allowed_hulls_for_role(&role_upper)
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();
        !allowed_hulls.contains(hull_name)
    } else {
        false // Not in fleet, can't check hull
    };
    
    // Add character to role (even if duplicate - frontend will handle confirmation)
    let role_assignments_entry = role_assignments.entry(role_upper.clone()).or_insert_with(Vec::new);
    // Always add, even if duplicate (frontend shows confirmation)
    role_assignments_entry.push(input.character_name.clone());
    
    // Filter out characters not in fleet from all roles
    for names in role_assignments.values_mut() {
        names.retain(|name| fleet_member_names.contains(&name.to_lowercase()));
    }
    
    // Reconstruct MOTD - add the new character to the role
    // We only modify the specific character's name and ID, preserving all HTML structure
    let new_motd = add_character_to_motd_role(
        &current_motd,
        &role_upper,
        &input.character_name,
        *character_name_to_id.get(&character_name_lower).unwrap_or(&0),
        &character_name_to_id,
        &fleet_member_names,
    );
    
    // Remove characters not in fleet - but do it surgically, preserving HTML structure
    let new_motd = remove_characters_not_in_fleet_surgical(
        &new_motd,
        &fleet_member_names,
    );
    
    // Update MOTD via ESI
    crate::core::esi::fleet_info::update(&app.esi_client, fleet_id, fleet.boss_id, new_motd).await?;
    
    Ok(Json(UpdateMotdResponse {
        success: true,
        message: "MOTD updated successfully".to_string(),
        is_duplicate,
        has_incorrect_hull,
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        fleet_status,
        fleet_info,
        close_fleet,
        fleet_members,
        register_fleet,
        fleet_composition,
        update_motd,
    ]
}
