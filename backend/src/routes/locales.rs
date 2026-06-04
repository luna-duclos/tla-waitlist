use std::collections::HashMap;

use rocket::serde::json::Json;
use serde::Serialize;
use serde_json::Value;

use crate::data::locales;
use crate::util::madness::Madness;

#[derive(Serialize)]
struct LanguagesResponse {
    languages: Vec<String>,
    labels: HashMap<String, String>,
}

#[get("/api/v2/locales/languages")]
fn list_languages() -> Json<LanguagesResponse> {
    let languages = locales::list_valid_languages();
    let labels = locales::language_labels_for_languages(&languages);
    Json(LanguagesResponse { languages, labels })
}

#[get("/api/v2/locales/<page>/<locale>")]
fn get_locale(page: String, locale: String) -> Result<Json<Value>, Madness> {
    let value = locales::read_locale_json(&page, &locale)?;
    Ok(Json(value))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![list_languages, get_locale]
}
