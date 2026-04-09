use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::Form as HtmlForm;
use eyre::WrapErr;
use maud::Markup;
use uuid::Uuid;

use crate::{
    db,
    error::AppError,
    japanese::conjugation::{self, ConjugationForm, Formality},
    srs::scheduler::{self, Rating},
    state::SharedState,
    templates::drill as tmpl,
};

pub async fn setup_form(State(state): State<SharedState>) -> Result<Markup, AppError> {
    let decks = db::decks::list_with_stats(&state.pool)
        .await
        .wrap_err("listing decks for drill setup")?;
    Ok(tmpl::setup(&decks))
}

#[derive(serde::Deserialize)]
pub struct StartDrillForm {
    #[serde(default)]
    pub deck_id: Option<i32>,
    #[serde(default)]
    pub forms: Vec<String>,
    #[serde(default)]
    pub formalities: Vec<String>,
}

pub async fn start_session(
    State(state): State<SharedState>,
    HtmlForm(input): HtmlForm<StartDrillForm>,
) -> Result<impl IntoResponse, AppError> {
    if input.forms.is_empty() || input.formalities.is_empty() {
        return Ok(Redirect::to("/drill/setup").into_response());
    }

    let session_id =
        db::sessions::create(&state.pool, input.deck_id, &input.forms, &input.formalities)
            .await
            .wrap_err("creating drill session")?;

    let due_count =
        db::drill_items::count_due(&state.pool, input.deck_id, &input.forms, &input.formalities)
            .await
            .wrap_err("counting due items")?;
    db::sessions::set_total(&state.pool, session_id, due_count.min(100) as i32)
        .await
        .wrap_err("setting session total")?;

    Ok(Redirect::to(&format!("/drill/{session_id}")).into_response())
}

pub async fn session_page(
    State(state): State<SharedState>,
    Path(session_id): Path<Uuid>,
) -> Result<Markup, AppError> {
    let session = db::sessions::get(&state.pool, session_id)
        .await
        .wrap_err("fetching drill session")?;
    Ok(tmpl::session_page(&session))
}

pub async fn current_card(
    State(state): State<SharedState>,
    Path(session_id): Path<Uuid>,
) -> Result<Markup, AppError> {
    let session = db::sessions::get(&state.pool, session_id)
        .await
        .wrap_err("fetching drill session")?;
    let due_items = scheduler::fetch_due_items(
        &state.pool,
        session.deck_id,
        &session.forms,
        &session.formalities,
        20,
    )
    .await
    .wrap_err("fetching due items")?;

    for item in &due_items {
        let form = match ConjugationForm::from_db_name(&item.form) {
            Some(f) => f,
            None => continue,
        };
        let formality = match Formality::from_db_name(&item.formality) {
            Some(f) => f,
            None => continue,
        };
        let verb_type = match item.verb_type.as_str() {
            "godan" => jp_inflections::VerbType::Godan,
            "ichidan" => jp_inflections::VerbType::Ichidan,
            "exception" => jp_inflections::VerbType::Exception,
            _ => continue,
        };

        let answers = match conjugation::conjugate(
            item.kanji.as_deref(),
            &item.kana,
            verb_type,
            form,
            formality,
        ) {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!(
                    verb = item.kana,
                    form = item.form,
                    error = %e,
                    "skipping unconjugable verb"
                );
                continue;
            }
        };

        return Ok(tmpl::card_frame(
            item,
            &session_id,
            form,
            formality,
            &answers,
        ));
    }

    let done = session.correct_count + session.wrong_count;
    Ok(tmpl::session_complete(
        done,
        session.correct_count,
        session.wrong_count,
    ))
}

#[derive(serde::Deserialize)]
pub struct AnswerForm {
    pub drill_item_id: i32,
    pub user_answer: String,
    pub correct_answers_json: String,
    #[serde(default, deserialize_with = "deserialize_optional_i64")]
    pub started_at_ms: i64,
}

fn deserialize_optional_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Ok(s.parse().unwrap_or(0))
}

pub async fn submit_answer(
    State(state): State<SharedState>,
    Path(session_id): Path<Uuid>,
    Form(input): Form<AnswerForm>,
) -> Result<Markup, AppError> {
    let correct_answers: Vec<String> =
        serde_json::from_str(&input.correct_answers_json).unwrap_or_default();
    let normalized = input.user_answer.trim();
    let is_correct = correct_answers.iter().any(|a| a == normalized);

    let rating = if is_correct {
        Rating::Good
    } else {
        Rating::Again
    };

    let item = db::drill_items::get_with_verb(&state.pool, input.drill_item_id)
        .await
        .wrap_err("fetching drill item with verb")?;
    let days_elapsed = item
        .last_review
        .map(|lr| (chrono::Utc::now() - lr).num_days().max(0) as u32)
        .unwrap_or(0);

    let schedule = scheduler::compute_next_review(
        &state.fsrs_parameters,
        item.stability,
        item.difficulty,
        state.config.desired_retention,
        days_elapsed,
        rating,
    );

    scheduler::update_drill_item_after_review(
        &state.pool,
        input.drill_item_id,
        &schedule,
        rating,
        chrono::Utc::now(),
    )
    .await
    .wrap_err("updating drill item after review")?;

    let response_ms = if input.started_at_ms > 0 {
        Some((chrono::Utc::now().timestamp_millis() - input.started_at_ms) as i32)
    } else {
        None
    };

    db::sessions::log_review(
        &state.pool,
        &db::sessions::ReviewEntry {
            session_id,
            drill_item_id: input.drill_item_id,
            user_answer: normalized,
            correct_answers: &correct_answers,
            is_correct,
            rating: rating as u8 as i16,
            response_ms,
        },
    )
    .await
    .wrap_err("logging review")?;

    db::sessions::increment_counts(&state.pool, session_id, is_correct)
        .await
        .wrap_err("incrementing session counts")?;

    let form = conjugation::ConjugationForm::from_db_name(&item.form);
    let formality = conjugation::Formality::from_db_name(&item.formality);

    let task_display = {
        let verb = item.kanji.as_deref().unwrap_or(&item.kana);
        let form_name = form.map(|f| f.short_name()).unwrap_or(&item.form);
        let formality_str = match formality {
            Some(conjugation::Formality::Plain) => " (plain)",
            Some(conjugation::Formality::Polite) => " (polite)",
            None => "",
        };
        format!("{verb} → {form_name}{formality_str}")
    };
    let verb_type = match item.verb_type.as_str() {
        "godan" => Some(jp_inflections::VerbType::Godan),
        "ichidan" => Some(jp_inflections::VerbType::Ichidan),
        "exception" => Some(jp_inflections::VerbType::Exception),
        _ => None,
    };
    let explanation = match (verb_type, form, formality) {
        (Some(vt), Some(f), Some(fm)) => Some(conjugation::explain(vt, f, fm)),
        _ => None,
    };
    let stem = verb_type.map(|vt| conjugation::masu_stem(&item.kana, vt));

    Ok(tmpl::answer_result_frame(&tmpl::AnswerContext {
        is_correct,
        user_answer: normalized,
        correct_answers: &correct_answers,
        session_id: &session_id,
        meaning: &item.meaning,
        task_display: &task_display,
        verb_kana: &item.kana,
        stem: stem.as_deref(),
        explanation: explanation.as_deref(),
    }))
}
