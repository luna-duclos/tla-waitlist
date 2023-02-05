use crate::data::yamlhelper;
use crate::util::madness::Madness;
use eve_data_core::TypeID;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct DNAFitting {
    pub name: String,
    pub dna: String,
}
#[derive(Debug, Serialize)]
struct FittingResponse {
    fittingdata: Option<Vec<DNAFitting>>,
    notes: Option<Vec<FittingNote>>,
    rules: Option<Vec<TypeID>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FittingNote {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct NoteFile {
    notes: Vec<FittingNote>,
}

fn load_notes_from_file() -> Vec<FittingNote> {
    let file: NoteFile = yamlhelper::from_file("./data/fitnotes.yaml");
    file.notes
}

#[get("/api/fittings")]
async fn fittings() -> Result<Json<FittingResponse>, Madness> {
    let fits = crate::data::fits::get_fits()
        .values()
        .flatten()
        .filter(|fit| !fit.name.contains("ALTERNATIVE"))
        .map(|fit| DNAFitting {
            name: fit.name.clone(),
            dna: fit.fit.to_dna().unwrap().clone(),
        })
        .collect::<Vec<_>>();

    let mut logirules = Vec::new();

    for rule in crate::data::categories::rules() {
        if rule.1 == "logi" {
            logirules.push(rule.0)
        }
    }
    Ok(Json(FittingResponse {
        fittingdata: Some(fits),
        notes: Some(load_notes_from_file()),
        rules: Some(logirules),
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![fittings]
}
