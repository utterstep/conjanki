use axum::Router;
use tower_http::services::ServeDir;

use crate::state::SharedState;

mod dashboard;
mod drill;
mod stats;
mod sync;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/", axum::routing::get(dashboard::index))
        .route("/sync/anki", axum::routing::post(sync::anki_sync))
        .route("/sync/data", axum::routing::post(sync::fetch_data))
        .route("/drill/setup", axum::routing::get(drill::setup_form))
        .route("/drill/start", axum::routing::post(drill::start_session))
        .route(
            "/drill/{session_id}",
            axum::routing::get(drill::session_page),
        )
        .route(
            "/drill/{session_id}/card",
            axum::routing::get(drill::current_card),
        )
        .route(
            "/drill/{session_id}/answer",
            axum::routing::post(drill::submit_answer),
        )
        .route("/stats", axum::routing::get(stats::index))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
