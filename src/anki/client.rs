use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};

use super::models::*;

pub struct AnkiClient {
    client: Client,
    base_url: String,
}

impl AnkiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    async fn request<P: Serialize, R: DeserializeOwned>(
        &self,
        action: &str,
        params: Option<P>,
    ) -> Result<Option<R>, crate::error::AppError> {
        let req = AnkiRequest {
            action: action.to_string(),
            version: 6,
            params,
        };
        let resp: AnkiResponse<R> = self
            .client
            .post(&self.base_url)
            .json(&req)
            .send()
            .await
            .map_err(|e| crate::error::AppError::Anki(e.to_string()))?
            .json()
            .await
            .map_err(|e| crate::error::AppError::Anki(e.to_string()))?;

        if let Some(err) = resp.error {
            return Err(crate::error::AppError::Anki(err));
        }
        Ok(resp.result)
    }

    /// Convenience wrapper for actions that must return a value.
    async fn request_value<P: Serialize, R: DeserializeOwned>(
        &self,
        action: &str,
        params: Option<P>,
    ) -> Result<R, crate::error::AppError> {
        self.request(action, params)
            .await?
            .ok_or_else(|| crate::error::AppError::Anki("null result".into()))
    }

    pub async fn sync(&self) -> Result<(), crate::error::AppError> {
        self.request::<(), ()>("sync", None).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn deck_names(&self) -> Result<Vec<String>, crate::error::AppError> {
        self.request_value::<(), _>("deckNames", None).await
    }

    pub async fn find_cards(&self, query: &str) -> Result<Vec<i64>, crate::error::AppError> {
        self.request_value(
            "findCards",
            Some(FindCardsParams {
                query: query.into(),
            }),
        )
        .await
    }

    pub async fn cards_info(&self, cards: &[i64]) -> Result<Vec<CardInfo>, crate::error::AppError> {
        self.request_value(
            "cardsInfo",
            Some(CardsInfoParams {
                cards: cards.to_vec(),
            }),
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn get_deck_stats(
        &self,
        decks: &[String],
    ) -> Result<serde_json::Value, crate::error::AppError> {
        self.request_value(
            "getDeckStats",
            Some(GetDeckStatsParams {
                decks: decks.to_vec(),
            }),
        )
        .await
    }
}
