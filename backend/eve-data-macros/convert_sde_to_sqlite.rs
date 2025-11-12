// Standalone script to convert official CCP SDE JSON files to SQLite database
// This replaces the fuzzwork SQLite download with direct SDE conversion

use rusqlite::{Connection, params};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const REQUIRED_ATTRIBUTE_IDS: &[i32] = &[
    275,  // skill training multiplier
    633,  // meta level
    984, 985, 986, 987,  // resists
    182, 183, 184, 1285, 1289, 1290,  // skill req
    277, 278, 279, 1286, 1287, 1288,  // skill req level
];

const REQUIRED_EFFECT_IDS: &[i32] = &[11, 12, 13, 2663];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let sde_dir = if args.len() > 1 {
        &args[1]
    } else {
        eprintln!("Usage: convert_sde_to_sqlite <sde_extracted_directory>");
        eprintln!("Example: convert_sde_to_sqlite /tmp/sde-2024.11");
        std::process::exit(1);
    };

    let output_file = "sqlite-shrunk.sqlite";
    
    println!("Converting SDE from {} to {}", sde_dir, output_file);
    
    // Remove existing database
    if Path::new(output_file).exists() {
        fs::remove_file(output_file)?;
    }

    // Create SQLite database
    let conn = Connection::open(output_file)?;
    
    // Create tables
    conn.execute(
        "CREATE TABLE invTypes (
            typeID INTEGER PRIMARY KEY,
            typeName TEXT NOT NULL,
            groupID INTEGER NOT NULL,
            published INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE invGroups (
            groupID INTEGER PRIMARY KEY,
            categoryID INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE invMetaTypes (
            typeID INTEGER NOT NULL,
            parentTypeID INTEGER,
            metaGroupID INTEGER
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE dgmTypeAttributes (
            typeID INTEGER NOT NULL,
            attributeID INTEGER NOT NULL,
            valueInt INTEGER,
            valueFloat REAL,
            PRIMARY KEY (typeID, attributeID)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE dgmTypeEffects (
            typeID INTEGER NOT NULL,
            effectID INTEGER NOT NULL,
            PRIMARY KEY (typeID, effectID)
        )",
        [],
    )?;

    // Process SDE files
    // Files are directly in sde_dir, not in an fsd/ subdirectory
    let sde_path = PathBuf::from(sde_dir);
    
    println!("Processing invTypes...");
    process_types(&conn, &sde_path)?;
    
    println!("Processing invGroups...");
    process_groups(&conn, &sde_path)?;
    
    println!("Processing invMetaTypes...");
    process_meta_types(&conn, &sde_path)?;
    
    println!("Processing dgmTypeAttributes and dgmTypeEffects...");
    process_type_dogma(&conn, &sde_path)?;

    // Create indexes
    println!("Creating indexes...");
    conn.execute("CREATE INDEX invTypes_name ON invTypes (typeName)", [])?;
    conn.execute("CREATE INDEX invTypes_typeID ON invTypes (typeID)", [])?;
    conn.execute("CREATE INDEX invGroups_groupID ON invGroups (groupID)", [])?;
    conn.execute("CREATE INDEX invMetaTypes_typeID ON invMetaTypes (typeID)", [])?;
    conn.execute("CREATE INDEX invMetaTypes_parentTypeID ON invMetaTypes (parentTypeID)", [])?;
    conn.execute("CREATE INDEX dgmTypeAttributes_typeID ON dgmTypeAttributes (typeID)", [])?;
    conn.execute("CREATE INDEX dgmTypeEffects_typeID ON dgmTypeEffects (typeID)", [])?;

    println!("✓ Conversion complete! Database saved to: {}", output_file);
    Ok(())
}

fn process_types(conn: &Connection, sde_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = sde_dir.join("types.jsonl");
    if !file_path.exists() {
        return Err(format!("File not found: {}", file_path.display()).into());
    }

    let file = fs::File::open(&file_path)?;
    let reader = BufReader::new(file);
    let mut stmt = conn.prepare(
        "INSERT INTO invTypes (typeID, typeName, groupID, published) VALUES (?1, ?2, ?3, ?4)"
    )?;

    let mut count = 0;
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let json: Value = serde_json::from_str(&line)?;
        
        let type_id: i32 = json["_key"].as_i64()
            .ok_or("Missing _key in type entry")? as i32;
        
        let name_obj = json.get("name")
            .ok_or("Missing name in type entry")?;
        let type_name = name_obj["en"]
            .as_str()
            .ok_or("Missing name.en in type entry")?
            .to_string();
        
        let group_id: i32 = json["groupID"]
            .as_i64()
            .ok_or("Missing groupID in type entry")? as i32;
        
        let published = json.get("published")
            .and_then(|v| v.as_bool())
            .unwrap_or(false) as i32;

        stmt.execute(params![type_id, type_name, group_id, published])?;
        count += 1;

        if count % 10000 == 0 {
            println!("  Processed {} types...", count);
        }
    }

    println!("  Inserted {} types", count);
    Ok(())
}

fn process_groups(conn: &Connection, sde_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = sde_dir.join("groups.jsonl");
    if !file_path.exists() {
        return Err(format!("File not found: {}", file_path.display()).into());
    }

    let file = fs::File::open(&file_path)?;
    let reader = BufReader::new(file);
    let mut stmt = conn.prepare(
        "INSERT INTO invGroups (groupID, categoryID) VALUES (?1, ?2)"
    )?;

    let mut count = 0;
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let json: Value = serde_json::from_str(&line)?;
        
        let group_id: i32 = json["_key"].as_i64()
            .ok_or("Missing _key in group entry")? as i32;
        
        let category_id: i32 = json["categoryID"]
            .as_i64()
            .ok_or("Missing categoryID in group entry")? as i32;

        stmt.execute(params![group_id, category_id])?;
        count += 1;
    }

    println!("  Inserted {} groups", count);
    Ok(())
}

fn process_meta_types(conn: &Connection, sde_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Meta types are stored in types.jsonl
    // We need to extract entries that have either:
    // 1. variationParentTypeID (child -> parent relationship)
    // 2. metaGroupID (all types with a meta group)
    let file_path = sde_dir.join("types.jsonl");
    if !file_path.exists() {
        println!("  types.jsonl not found, skipping meta types...");
        return Ok(());
    }

    let file = fs::File::open(&file_path)?;
    let reader = BufReader::new(file);
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO invMetaTypes (typeID, parentTypeID, metaGroupID) VALUES (?1, ?2, ?3)"
    )?;

    let mut count = 0;
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let json: Value = serde_json::from_str(&line)?;
        
        let type_id: i32 = json["_key"].as_i64()
            .ok_or("Missing _key in type entry")? as i32;
        
        // Get metaGroupID if it exists
        let meta_group_id: Option<i32> = json.get("metaGroupID")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        
        // Get variationParentTypeID if it exists
        let parent_type_id: Option<i32> = json.get("variationParentTypeID")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        
        // Insert entry: if there's a parent, use it; otherwise parent is NULL
        // The old database seems to have entries for all types with metaGroupID
        if meta_group_id.is_some() || parent_type_id.is_some() {
            stmt.execute(params![type_id, parent_type_id, meta_group_id])?;
            count += 1;
        }
    }

    println!("  Inserted {} meta types", count);
    Ok(())
}

fn process_type_dogma(conn: &Connection, sde_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = sde_dir.join("typeDogma.jsonl");
    if !file_path.exists() {
        return Err(format!("File not found: {}", file_path.display()).into());
    }

    let required_attrs: HashSet<i32> = REQUIRED_ATTRIBUTE_IDS.iter().copied().collect();
    let required_effects: HashSet<i32> = REQUIRED_EFFECT_IDS.iter().copied().collect();
    
    let file = fs::File::open(&file_path)?;
    let reader = BufReader::new(file);
    
    let mut attr_stmt = conn.prepare(
        "INSERT INTO dgmTypeAttributes (typeID, attributeID, valueInt, valueFloat) VALUES (?1, ?2, ?3, ?4)"
    )?;
    
    let mut effect_stmt = conn.prepare(
        "INSERT INTO dgmTypeEffects (typeID, effectID) VALUES (?1, ?2)"
    )?;

    let mut attr_count = 0;
    let mut effect_count = 0;
    
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let json: Value = serde_json::from_str(&line)?;
        
        let type_id: i32 = json["_key"].as_i64()
            .ok_or("Missing _key in typeDogma entry")? as i32;
        
        // Process dogmaAttributes
        if let Some(attributes) = json.get("dogmaAttributes").and_then(|v| v.as_array()) {
            for attr in attributes {
                let attribute_id: i32 = attr["attributeID"].as_i64()
                    .ok_or("Missing attributeID in dogmaAttributes")? as i32;
                
                // Only process required attributes
                if !required_attrs.contains(&attribute_id) {
                    continue;
                }
                
                // Handle value - can be integer or float
                let value = attr.get("value")
                    .ok_or("Missing value in dogmaAttributes")?;
                
                let value_int: Option<i32> = if value.is_i64() {
                    Some(value.as_i64().unwrap() as i32)
                } else if value.is_f64() {
                    let fval = value.as_f64().unwrap();
                    // Only store as int if it's a whole number
                    if fval.fract() == 0.0 {
                        Some(fval as i32)
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                let value_float: Option<f64> = if value.is_f64() {
                    Some(value.as_f64().unwrap())
                } else if value.is_i64() {
                    Some(value.as_i64().unwrap() as f64)
                } else {
                    None
                };

                attr_stmt.execute(params![type_id, attribute_id, value_int, value_float])?;
                attr_count += 1;
            }
        }
        
        // Process dogmaEffects
        if let Some(effects) = json.get("dogmaEffects").and_then(|v| v.as_array()) {
            for effect in effects {
                let effect_id: i32 = effect["effectID"].as_i64()
                    .ok_or("Missing effectID in dogmaEffects")? as i32;
                
                // Only process required effects
                if !required_effects.contains(&effect_id) {
                    continue;
                }

                effect_stmt.execute(params![type_id, effect_id])?;
                effect_count += 1;
            }
        }
    }

    println!("  Inserted {} type attributes", attr_count);
    println!("  Inserted {} type effects", effect_count);
    Ok(())
}

