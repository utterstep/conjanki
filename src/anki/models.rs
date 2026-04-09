use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AnkiRequest<T: Serialize> {
    pub action: String,
    pub version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
}

#[derive(Deserialize)]
pub struct AnkiResponse<T> {
    pub result: Option<T>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct FindCardsParams {
    pub query: String,
}

#[derive(Serialize)]
pub struct CardsInfoParams {
    pub cards: Vec<i64>,
}

#[derive(Serialize)]
#[allow(dead_code)]
pub struct GetDeckStatsParams {
    pub decks: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // fields populated by deserialization
pub struct CardInfo {
    #[serde(rename = "cardId")]
    pub card_id: i64,
    #[serde(rename = "deckName")]
    pub deck_name: String,
    #[serde(rename = "modelName")]
    pub model_name: String,
    pub fields: serde_json::Map<String, serde_json::Value>,
}

impl CardInfo {
    pub fn field_text(&self, name: &str) -> Option<String> {
        let field = self.fields.get(name)?;
        let value = field.get("value")?.as_str()?;
        // Strip HTML tags
        let stripped = strip_html(value);
        let trimmed = stripped.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }
}

fn strip_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    // Also decode &nbsp;
    result.replace("&nbsp;", " ")
}
