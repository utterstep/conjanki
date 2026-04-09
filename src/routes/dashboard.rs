use axum::extract::State;
use eyre::WrapErr;
use maud::Markup;

use crate::{db, error::AppError, state::SharedState, templates};

pub async fn index(State(state): State<SharedState>) -> Result<Markup, AppError> {
    let decks = db::decks::list_with_stats(&state.pool)
        .await
        .wrap_err("listing decks with stats")?;
    Ok(templates::dashboard::render(&decks, None))
}
