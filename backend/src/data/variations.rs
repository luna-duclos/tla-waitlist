use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::data::yamlhelper;
use crate::util::madness::Madness;

use eve_data_core::{Attribute, TypeDB, TypeError, TypeID};

lazy_static::lazy_static! {
    static ref INSTANCE: Arc<RwLock<Variator>> = Arc::new(RwLock::new(Builder::build().unwrap()));
}

#[derive(Debug)]
pub struct Variation {
    pub from: TypeID,
    pub to: TypeID,
    pub meta_diff: i64,
}

#[derive(Debug)]
pub struct Variator {
    variations: BTreeMap<TypeID, Vec<Variation>>,
    pub cargo_ignore: BTreeSet<TypeID>,
}

impl Variator {
    pub fn get(&self, from: TypeID) -> Option<&[Variation]> {
        match self.variations.get(&from) {
            Some(vars) => Some(vars),
            None => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct FromMetaEntry {
    base: String,
    abyssal: Option<String>,
    alternative: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FromAttributeEntry {
    base: Vec<String>,
    attribute: i32,
    reverse: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModuleFile {
    alternatives: Vec<Vec<Vec<String>>>,
    from_meta: Vec<FromMetaEntry>,
    from_attribute: Vec<FromAttributeEntry>,
    accept_t1: Vec<String>,
    cargo_ignore: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModuleString {
    name: String,
    amount: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct AddRemove {
    detect: String,
    remove: Vec<String>,
    add: Vec<ModuleString>,
}

#[derive(Debug)]
pub struct DrugChanger {
    pub add: BTreeMap<TypeID, i64>,
    pub remove: BTreeSet<TypeID>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModuleFileDrug {
    drugs_approve_override: Vec<AddRemove>,
}

struct Builder {
    variations: BTreeMap<TypeID, Vec<Variation>>,
    cargo_ignore: BTreeSet<TypeID>,
    file: ModuleFile,
}

impl Builder {
    fn build() -> Result<Variator, TypeError> {
        let mut builder = Builder {
            variations: BTreeMap::new(),
            cargo_ignore: BTreeSet::new(),
            file: yamlhelper::from_file("./data/modules.yaml"),
        };
        builder.add_alternatives()?;
        builder.add_meta()?;
        builder.add_t1()?;
        builder.add_by_attribute()?;
        builder.add_cargo_ignore()?;

        Ok(Variator {
            variations: builder.variations,
            cargo_ignore: builder.cargo_ignore,
        })
    }

    fn merge_tiers(&mut self, tiers: HashMap<TypeID, i64>) {
        for (&module_i, &tier_i) in &tiers {
            if self.variations.contains_key(&module_i) {
                panic!("Duplicate declaration for ID {}", module_i);
            }
            let mut vars = Vec::new();

            for (&module_j, &tier_j) in &tiers {
                vars.push(Variation {
                    from: module_i,
                    to: module_j,
                    meta_diff: tier_j - tier_i,
                });
            }

            vars.sort_by_key(|v| {
                if v.meta_diff < 0 {
                    1000000 - v.meta_diff
                } else {
                    v.meta_diff
                }
            });

            self.variations.insert(module_i, vars);
        }
    }

    fn add_alternatives(&mut self) -> Result<(), TypeError> {
        let mut to_merge = vec![];
        for group in &self.file.alternatives {
            let mut tiers = HashMap::new();
            let mut tier_i = 0;
            for tier in group {
                tier_i += 1;
                for module in tier {
                    tiers.insert(TypeDB::id_of(module)?, tier_i);
                }
            }
            to_merge.push(tiers);
        }
        for merge in to_merge {
            self.merge_tiers(merge);
        }
        Ok(())
    }
    fn add_cargo_ignore(&mut self) -> Result<(), TypeError> {
        for entry in &self.file.cargo_ignore {
            self.cargo_ignore.insert(TypeDB::id_of(entry)?);
        }
        Ok(())
    }

    fn add_meta(&mut self) -> Result<(), TypeError> {
        let mut to_merge = vec![];
        for entry in &self.file.from_meta {
            let base_id = TypeDB::id_of(&entry.base)?;
            let mut variations = TypeDB::type_variations(base_id)?;
            if let Some(abyssal) = &entry.abyssal {
                variations.insert(TypeDB::id_of(abyssal)?, 17);
            }

            if let Some(alternative) = &entry.alternative {
                variations.insert(TypeDB::id_of(alternative)?, 17);
            }
            to_merge.push(variations);
        }
        for merge in to_merge {
            self.merge_tiers(merge);
        }
        Ok(())
    }

    fn add_t1(&mut self) -> Result<(), TypeError> {
        let mut to_merge = vec![];
        for entry in &self.file.accept_t1 {
            let mut tiers = HashMap::new();
            tiers.insert(TypeDB::id_of(entry)?, 2);
            tiers.insert(TypeDB::id_of(&entry[..entry.len() - 1])?, 1);
            to_merge.push(tiers);
        }
        for merge in to_merge {
            self.merge_tiers(merge);
        }
        Ok(())
    }

    fn add_by_attribute(&mut self) -> Result<(), TypeError> {
        let mut to_merge = vec![];

        for entry in &self.file.from_attribute {
            let attribute = Attribute::from_id(entry.attribute);

            let mut module_ids = Vec::new();
            for base in &entry.base {
                for (variation_id, _meta) in TypeDB::type_variations(TypeDB::id_of(base)?)? {
                    module_ids.push(variation_id);
                }
            }

            let mut modules_with_attribute = TypeDB::load_types(&module_ids)
                .unwrap()
                .into_iter()
                .map(|(type_id, the_type)| {
                    if let Some(attr) = the_type.unwrap().attributes.get(&attribute) {
                        (type_id, *attr)
                    } else {
                        panic!("Missing attribute {:?} for type {}", attribute, type_id);
                    }
                })
                .collect::<Vec<_>>();
            modules_with_attribute.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            if let Some(true) = entry.reverse {
                modules_with_attribute.reverse();
            }

            let mut tiers = HashMap::new();
            let mut tier_i = 1;
            let mut last_value = modules_with_attribute.get(0).unwrap().1;
            for (type_id, attribute_value) in modules_with_attribute {
                if (last_value - attribute_value).abs() > 0.0000000001 {
                    tier_i += 1;
                    last_value = attribute_value;
                }

                tiers.insert(type_id, tier_i);
            }

            to_merge.push(tiers);
        }

        for merge in to_merge {
            self.merge_tiers(merge);
        }

        Ok(())
    }
}

pub fn drug_handling() -> Result<BTreeMap<TypeID, DrugChanger>, TypeError> {
    let data: ModuleFileDrug = yamlhelper::from_file("./data/modules.yaml");

    let mut drugmap = BTreeMap::<TypeID, DrugChanger>::new();

    for itemtype in &data.drugs_approve_override {
        let mut remove = BTreeSet::<TypeID>::new();
        let mut add = BTreeMap::<TypeID, i64>::new();
        for entry in &itemtype.remove {
            remove.insert(TypeDB::id_of(entry)?);
        }
        for entry in &itemtype.add {
            add.insert(TypeDB::id_of(&entry.name)?, entry.amount);
        }
        drugmap.insert(
            TypeDB::id_of(&itemtype.detect)?,
            DrugChanger {
                add: add,
                remove: remove,
            },
        );
    }
    Ok(drugmap)
}

#[derive(Debug, Deserialize, Serialize)]
struct FitVariationRule {
    hull: String,
    missing: Option<Vec<ModuleString>>,
    extra: Option<Vec<ModuleString>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FitVariationRules {
    fit_variation_rules: Vec<FitVariationRule>,
}

#[derive(Debug, Serialize)]
pub struct ModuleVariation {
    pub missing: BTreeMap<TypeID, i64>,
    pub extra: BTreeMap<TypeID, i64>,
}

pub fn fit_module_variations() -> Result<BTreeMap<TypeID, Vec<ModuleVariation>>, TypeError> {
    let data: FitVariationRules = yamlhelper::from_file("./data/modules.yaml");
    let mut hull_variations = BTreeMap::<TypeID, Vec<ModuleVariation>>::new();

    for rule in &data.fit_variation_rules {
        let mut missing = BTreeMap::<TypeID, i64>::new();
        let mut extra = BTreeMap::<TypeID, i64>::new();

        if let Some(item) = &rule.missing {
            for module in item {
                missing.insert(TypeDB::id_of(&module.name)?, module.amount);
            }
        }

        if let Some(item) = &rule.extra {
            for module in item {
                extra.insert(TypeDB::id_of(&module.name)?, module.amount);
            }
        }
        let hull_id = TypeDB::id_of(&rule.hull)?;
        if let Some(hull_var) = hull_variations.get_mut(&hull_id) {
            hull_var.push(ModuleVariation {
                missing: missing,
                extra: extra,
            });
        } else {
            hull_variations.insert(
                hull_id,
                vec![ModuleVariation {
                    missing: missing,
                    extra: extra,
                }],
            );
        }
    }

    Ok(hull_variations)
}

pub fn get() -> Arc<RwLock<Variator>> {
    INSTANCE.clone()
}

pub fn reload_variations() -> Result<(), TypeError> {
    let new_variator = Builder::build()?;
    *INSTANCE.write().unwrap() = new_variator;
    Ok(())
}

use std::fs;
use std::path::Path;

pub fn save_modules_to_file(yaml_content: &str) -> Result<(), Madness> {
    use std::io::Write;
    
    // Validate the YAML before saving
    validate_yaml(yaml_content)?;
    
    // Create backup
    create_backup()?;
    
    // Write to temporary file
    let temp_path = "./data/modules.yaml.tmp";
    let mut temp_file = fs::File::create(temp_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create temp file: {}", e)))?;
    
    temp_file.write_all(yaml_content.as_bytes())
        .map_err(|e| Madness::BadRequest(format!("Failed to write temp file: {}", e)))?;
    
    temp_file.sync_all()
        .map_err(|e| Madness::BadRequest(format!("Failed to sync temp file: {}", e)))?;
    
    // Atomic rename
    fs::rename(temp_path, "./data/modules.yaml")
        .map_err(|e| Madness::BadRequest(format!("Failed to rename temp file: {}", e)))?;
    
    Ok(())
}

pub fn validate_yaml(yaml_content: &str) -> Result<(), Madness> {
    // Validate YAML syntax and structure
    let _: ModuleFile = serde_yaml::from_str(yaml_content)
        .map_err(|e| Madness::BadRequest(format!("Invalid YAML: {}", e)))?;
    Ok(())
}

pub fn create_backup() -> Result<(), Madness> {
    use std::time::SystemTime;
    
    let source = "./data/modules.yaml";
    if !Path::new(source).exists() {
        return Ok(());
    }
    
    let timestamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let backup_path = format!("./data/modules.yaml.backup.{}", timestamp);
    
    fs::copy(source, &backup_path)
        .map_err(|e| Madness::BadRequest(format!("Failed to create backup: {}", e)))?;
    
    // Keep only last 5 backups
    let mut backups: Vec<_> = fs::read_dir("./data")
        .map_err(|e| Madness::BadRequest(format!("Failed to read data directory: {}", e)))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("modules.yaml.backup.") {
                if let Some(timestamp_str) = name.strip_prefix("modules.yaml.backup.") {
                    if let Ok(ts) = timestamp_str.parse::<u64>() {
                        return Some((entry.path(), ts));
                    }
                }
            }
            None
        })
        .collect();
    
    backups.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Remove old backups (keep last 5)
    for (path, _) in backups.into_iter().skip(5) {
        let _ = fs::remove_file(path);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{TypeDB, TypeID};

    #[derive(Debug)]
    enum Diff {
        Better,
        Worse,
        Equal,
    }

    fn id_of(s: &str) -> TypeID {
        TypeDB::id_of(s).unwrap()
    }

    fn test_diff(from: &str, to: &str, diff: Diff) {
        let variations = super::get()
            .get(id_of(from))
            .expect("Missing expected variation [from]");
        let to_id = id_of(to);
        let the_match = variations
            .iter()
            .find(|v| v.to == to_id)
            .expect("Missing expected variation [to]");
        match diff {
            Diff::Better => assert!(
                the_match.meta_diff > 0,
                "Expecting {:?}: {:#?}",
                diff,
                the_match
            ),
            Diff::Worse => assert!(
                the_match.meta_diff < 0,
                "Expecting {:?}: {:#?}",
                diff,
                the_match
            ),
            Diff::Equal => assert!(
                the_match.meta_diff == 0,
                "Expecting {:?}: {:#?}",
                diff,
                the_match
            ),
        }
    }

    #[test]
    fn test_a_few() {
        // Hardcoded ones
        test_diff(
            "Agency 'Pyrolancea' DB5 Dose II",
            "Agency 'Pyrolancea' DB3 Dose I",
            Diff::Worse,
        );
        test_diff(
            "Agency 'Pyrolancea' DB5 Dose II",
            "Agency 'Pyrolancea' DB7 Dose III",
            Diff::Better,
        );
        test_diff(
            "Agency 'Pyrolancea' DB5 Dose II",
            "Agency 'Pyrolancea' DB5 Dose II",
            Diff::Equal,
        );

        // T2->T1
        test_diff(
            "Heavy Hull Maintenance Bot II",
            "Heavy Hull Maintenance Bot I",
            Diff::Worse,
        );
        test_diff(
            "Heavy Hull Maintenance Bot I",
            "Heavy Hull Maintenance Bot II",
            Diff::Better,
        );
        test_diff(
            "Heavy Hull Maintenance Bot II",
            "Heavy Hull Maintenance Bot II",
            Diff::Equal,
        );

        // Meta
        test_diff(
            "Core X-Type 500MN Microwarpdrive",
            "Core C-Type 500MN Microwarpdrive",
            Diff::Worse,
        );
        test_diff(
            "500MN Microwarpdrive I",
            "Core C-Type 500MN Microwarpdrive",
            Diff::Better,
        );
        test_diff(
            "Gist X-Type 500MN Microwarpdrive",
            "Core X-Type 500MN Microwarpdrive",
            Diff::Equal,
        );

        // Attributes
        test_diff(
            "Centum A-Type Multispectrum Energized Membrane",
            "Centii A-Type Multispectrum Coating",
            Diff::Worse,
        );
        test_diff(
            "Centum A-Type Multispectrum Energized Membrane",
            "Corpum A-Type Multispectrum Energized Membrane",
            Diff::Equal,
        );
        test_diff(
            "Federation Navy Multispectrum Energized Membrane",
            "Multispectrum Energized Membrane II",
            Diff::Equal,
        );
    }
}
