use std::sync::Arc;

use lindera::tokenizer::Tokenizer;
use sqlx::PgPool;

use crate::config::{Config, DeckConfigs};

pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub deck_configs: DeckConfigs,
    pub tokenizer: Tokenizer,
    /// FSRS parameters for scheduling. We store the params and create FSRS on demand
    /// because fsrs::FSRS contains burn tensors that are not Send+Sync.
    pub fsrs_parameters: Vec<f32>,
}

pub type SharedState = Arc<AppState>;
