use std::sync::Arc;

use eyre::WrapErr;
use tracing_subscriber::EnvFilter;

mod anki;
mod config;
mod db;
mod error;
mod japanese;
mod routes;
mod srs;
mod state;
mod templates;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();
    let config: config::Config = envy::from_env().wrap_err("loading config from environment")?;
    let deck_configs =
        config::DeckConfigs::load("conjanki.toml").wrap_err("loading deck config")?;
    let bind_addr = config.bind_address.clone();

    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .wrap_err("connecting to database")?;
    sqlx::migrate!()
        .run(&pool)
        .await
        .wrap_err("running database migrations")?;

    let dictionary = lindera::dictionary::load_dictionary("embedded://ipadic")
        .wrap_err("loading ipadic dictionary")?;
    let segmenter =
        lindera::segmenter::Segmenter::new(lindera::mode::Mode::Normal, dictionary, None);
    let tokenizer = lindera::tokenizer::Tokenizer::new(segmenter);

    let fsrs_parameters = fsrs::DEFAULT_PARAMETERS.to_vec();

    let state = Arc::new(state::AppState {
        pool,
        config,
        deck_configs,
        tokenizer,
        fsrs_parameters,
    });

    let app = routes::build_router(state);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .wrap_err("binding tcp listener")?;
    tracing::info!("Listening on {}", bind_addr);
    axum::serve(listener, app)
        .await
        .wrap_err("running server")?;
    Ok(())
}
