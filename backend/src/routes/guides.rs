use std::fmt;

use rocket::http::ContentType;
use rocket::response::status;
use rocket::serde::json::Json;
use serde::Serialize;

use crate::data::guides;
use crate::util::madness::Madness;

#[derive(Serialize)]
struct GuidesListResponse {
    guides: Vec<guides::GuideListing>,
}

#[get("/api/v2/guides")]
fn list_guides() -> Json<GuidesListResponse> {
    Json(GuidesListResponse {
        guides: guides::list_guides(),
    })
}

#[get("/api/v2/guides/<slug>/assets/<filename>")]
fn get_guide_asset(
    slug: String,
    filename: String,
) -> Result<(ContentType, Vec<u8>), status::NotFound<String>> {
    let bytes = guides::read_asset(&slug, &filename).map_err(|e| match e {
        Madness::BadRequest(message) => status::NotFound(message),
        other => status::NotFound(fmt::format(format_args!("{other}"))),
    })?;
    let mime = guides::asset_mime_type(&filename).map_err(|e| match e {
        Madness::BadRequest(message) => status::NotFound(message),
        other => status::NotFound(fmt::format(format_args!("{other}"))),
    })?;
    let content_type = ContentType::parse_flexible(mime).unwrap_or(ContentType::Binary);
    Ok((content_type, bytes))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![list_guides, get_guide_asset]
}
