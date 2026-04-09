use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use maud::html;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("AnkiConnect error: {0}")]
    Anki(String),

    #[error("Conjugation error: {0}")]
    #[allow(dead_code)]
    Conjugation(String),

    #[error("Not found")]
    #[allow(dead_code)]
    NotFound,

    #[error("{0}")]
    Internal(#[from] eyre::Report),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = crate::templates::layout::page(
            "Error",
            html! {
                h1 { (status.as_str()) }
                p { (self.to_string()) }
            },
        );
        (status, body).into_response()
    }
}
