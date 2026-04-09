use axum::{extract::State, http::StatusCode};
use eyre::WrapErr;
use maud::{Markup, html};

use crate::{
    anki::client::AnkiClient,
    db,
    error::AppError,
    japanese::{
        analyzer,
        conjugation::{self, ConjugationForm, Formality},
    },
    state::SharedState,
    templates,
};

/// Step 1: trigger AnkiConnect to sync with AnkiWeb.
pub async fn anki_sync(State(state): State<SharedState>) -> Result<StatusCode, AppError> {
    let anki = AnkiClient::new(&state.config.anki_connect_url);
    anki.sync().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Step 2: fetch card data from local AnkiConnect and upsert into our DB.
pub async fn fetch_data(State(state): State<SharedState>) -> Result<Markup, AppError> {
    let anki = AnkiClient::new(&state.config.anki_connect_url);
    let mut total_verbs = 0u32;
    let mut total_decks = 0u32;

    for deck_config in &state.deck_configs.deck {
        let deck_id = db::decks::upsert(&state.pool, &deck_config.name)
            .await
            .wrap_err("upserting deck")?;

        let card_ids = anki.find_cards(&deck_config.query).await?;
        if card_ids.is_empty() {
            continue;
        }
        total_decks += 1;

        for chunk in card_ids.chunks(50) {
            let cards = anki.cards_info(chunk).await?;

            for card in &cards {
                let word = match card.field_text(&deck_config.fields.word) {
                    Some(w) => w,
                    None => continue,
                };
                let reading = card
                    .field_text(&deck_config.fields.reading)
                    .unwrap_or_default();
                let meaning = card
                    .field_text(&deck_config.fields.meaning)
                    .unwrap_or_else(|| "—".into());

                let analysis = match analyzer::analyze_verb(&state.tokenizer, &word) {
                    Some(a) => a,
                    None => continue,
                };

                let verb_type_str = match analysis.verb_type {
                    jp_inflections::VerbType::Godan => "godan",
                    jp_inflections::VerbType::Ichidan => "ichidan",
                    jp_inflections::VerbType::Exception => "exception",
                };

                let kanji = if word != reading {
                    Some(word.as_str())
                } else {
                    None
                };
                let kana = if reading.is_empty() { &word } else { &reading };

                // Verify jp_inflections can actually conjugate this word
                if conjugation::conjugate(
                    kanji,
                    kana,
                    analysis.verb_type,
                    ConjugationForm::Dictionary,
                    Formality::Plain,
                )
                .is_err()
                {
                    tracing::debug!(word = %word, "skipping non-conjugable word");
                    continue;
                }

                let verb_id = db::verbs::upsert(
                    &state.pool,
                    deck_id,
                    card.card_id,
                    kanji,
                    kana,
                    &meaning,
                    verb_type_str,
                )
                .await
                .wrap_err("upserting verb")?;

                for form in ConjugationForm::all() {
                    let formalities = if form.supports_formality() {
                        vec![Formality::Plain, Formality::Polite]
                    } else {
                        vec![Formality::Plain]
                    };
                    for formality in &formalities {
                        db::drill_items::upsert(
                            &state.pool,
                            verb_id,
                            form.db_name(),
                            formality.db_name(),
                        )
                        .await
                        .wrap_err("upserting drill item")?;
                    }
                }

                total_verbs += 1;
            }
        }

        db::decks::set_last_synced(&state.pool, deck_id)
            .await
            .wrap_err("setting deck last_synced")?;
    }

    tracing::info!("Synced {total_verbs} verbs across {total_decks} decks");

    let decks = db::decks::list_with_stats(&state.pool)
        .await
        .wrap_err("listing decks after sync")?;
    let flash = format!("Synced {total_verbs} verbs across {total_decks} decks.");

    Ok(html! {
        turbo-frame id="dashboard-content" {
            (templates::dashboard::dashboard_content(&decks, Some(&flash)))
        }
    })
}
