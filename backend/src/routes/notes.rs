use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{
    app,
    core::auth::AuthenticatedAccount,
    routes::waitlist::notify,
    util::{madness::Madness, types::Character},
};

#[derive(Debug, Serialize)]
struct NotesListNote {
    id: i64,
    author: Character,
    logged_at: i64,
    note: String,
    show_on_waitlist: bool,
}

#[derive(Debug, Serialize)]
struct NotesList {
    notes: Vec<NotesListNote>,
}

#[get("/api/notes?<character_id>")]
async fn list_notes(
    account: AuthenticatedAccount,
    character_id: i64,
    app: &rocket::State<app::Application>,
) -> Result<Json<NotesList>, Madness> {
    account.require_access("notes-view")?;

    let notes_q = sqlx::query!(
        "
            SELECT cn.id, cn.author_id, author.name author_name, cn.note, cn.logged_at, cn.show_on_waitlist
            FROM character_note cn
            JOIN `character` author ON author.id = cn.author_id
            WHERE cn.character_id = ?
            ORDER BY cn.logged_at ASC
        ",
        character_id
    )
    .fetch_all(app.get_db())
    .await?;
    let notes = notes_q
        .into_iter()
        .map(|note| NotesListNote {
            id: note.id,
            author: Character {
                id: note.author_id,
                name: note.author_name,
                corporation_id: None,
            },
            logged_at: note.logged_at,
            note: note.note,
            show_on_waitlist: note.show_on_waitlist > 0,
        })
        .collect();

    Ok(Json(NotesList { notes }))
}

#[derive(Deserialize)]
struct AddNoteInput {
    character_id: i64,
    note: String,
}

#[post("/api/notes/add", data = "<input>")]
async fn add_note(
    account: AuthenticatedAccount,
    app: &rocket::State<app::Application>,
    input: Json<AddNoteInput>,
) -> Result<&'static str, Madness> {
    account.require_access("notes-add")?;

    if input.note.len() < 10 || input.note.len() > 5000 {
        return Err(Madness::BadRequest("Invalid note".to_string()));
    }

    let now = chrono::Utc::now().timestamp();
    sqlx::query!(
        "INSERT INTO character_note (author_id, character_id, note, logged_at) VALUES (?, ?, ?, ?)",
        account.id,
        input.character_id,
        input.note,
        now,
    )
    .execute(app.get_db())
    .await?;

    Ok("OK")
}

#[derive(Deserialize)]
struct ToggleWaitlistNoteInput {
    note_id: i64,
    show_on_waitlist: bool,
}

#[post("/api/notes/toggle_waitlist", data = "<input>")]
async fn toggle_waitlist_note(
    account: AuthenticatedAccount,
    app: &rocket::State<app::Application>,
    input: Json<ToggleWaitlistNoteInput>,
) -> Result<&'static str, Madness> {
    account.require_access("notes-add")?;

    let note = sqlx::query!(
        "SELECT id, character_id FROM character_note WHERE id = ?",
        input.note_id
    )
    .fetch_optional(app.get_db())
    .await?;

    let note = note.ok_or_else(|| Madness::BadRequest("Note not found".to_string()))?;

    if input.show_on_waitlist {
        sqlx::query!(
            "UPDATE character_note SET show_on_waitlist = 0 WHERE character_id = ?",
            note.character_id
        )
        .execute(app.get_db())
        .await?;
    }

    let show_on_waitlist = if input.show_on_waitlist { 1 } else { 0 };
    sqlx::query!(
        "UPDATE character_note SET show_on_waitlist = ? WHERE id = ?",
        show_on_waitlist,
        input.note_id
    )
    .execute(app.get_db())
    .await?;

    let waitlists = sqlx::query!(
        "
            SELECT we.waitlist_id
            FROM waitlist_entry we
            JOIN waitlist w ON w.id = we.waitlist_id AND w.is_open = 1
            WHERE we.account_id = ?
        ",
        note.character_id
    )
    .fetch_all(app.get_db())
    .await?;

    for waitlist in waitlists {
        notify::notify_waitlist_update(app, waitlist.waitlist_id).await?;
    }

    Ok("OK")
}

pub fn routes() -> Vec<rocket::Route> {
    routes![list_notes, add_note, toggle_waitlist_note]
}
