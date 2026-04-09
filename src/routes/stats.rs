use axum::extract::State;
use eyre::WrapErr;
use maud::Markup;

use crate::{db, error::AppError, state::SharedState, templates};

pub async fn index(State(state): State<SharedState>) -> Result<Markup, AppError> {
    let deck_stats = db::decks::list_with_stats(&state.pool)
        .await
        .wrap_err("listing decks with stats")?;
    let form_accuracy = db::sessions::accuracy_by_form(&state.pool)
        .await
        .wrap_err("fetching accuracy by form")?;
    let weak_verbs = db::sessions::weakest_verbs(&state.pool, 10)
        .await
        .wrap_err("fetching weakest verbs")?;
    let recent_sessions = db::sessions::recent(&state.pool, 10)
        .await
        .wrap_err("fetching recent sessions")?;
    Ok(templates::stats::render(
        &deck_stats,
        &form_accuracy,
        &weak_verbs,
        &recent_sessions,
    ))
}
