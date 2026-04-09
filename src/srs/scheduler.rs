use chrono::{DateTime, Utc};
use fsrs::{FSRS, MemoryState};
use sqlx::PgPool;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)] // Hard and Easy are standard FSRS ratings
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

pub struct ScheduleResult {
    pub stability: f32,
    pub difficulty: f32,
    pub interval_days: f32,
}

pub fn compute_next_review(
    fsrs_params: &[f32],
    current_stability: Option<f32>,
    current_difficulty: Option<f32>,
    desired_retention: f32,
    days_since_last_review: u32,
    rating: Rating,
) -> ScheduleResult {
    let fsrs = FSRS::new(Some(fsrs_params)).expect("valid FSRS params");

    let memory_state = match (current_stability, current_difficulty) {
        (Some(s), Some(d)) => Some(MemoryState {
            stability: s,
            difficulty: d,
        }),
        _ => None,
    };

    let next_states = fsrs
        .next_states(memory_state, desired_retention, days_since_last_review)
        .expect("FSRS next_states failed");

    let chosen = match rating {
        Rating::Again => &next_states.again,
        Rating::Hard => &next_states.hard,
        Rating::Good => &next_states.good,
        Rating::Easy => &next_states.easy,
    };

    ScheduleResult {
        stability: chosen.memory.stability,
        difficulty: chosen.memory.difficulty,
        interval_days: chosen.interval,
    }
}

pub async fn update_drill_item_after_review(
    pool: &PgPool,
    drill_item_id: i32,
    result: &ScheduleResult,
    rating: Rating,
    now: DateTime<Utc>,
) -> sqlx::Result<()> {
    let due = now + chrono::Duration::seconds((result.interval_days * 86400.0) as i64);
    let is_lapse = matches!(rating, Rating::Again);

    let state = if result.interval_days < 1.0 {
        "learning"
    } else {
        "review"
    };

    sqlx::query!(
        r#"
        UPDATE drill_items SET
            stability = $1,
            difficulty = $2,
            due = $3,
            last_interval = $4,
            lapses = lapses + CASE WHEN $5 THEN 1 ELSE 0 END,
            reps = reps + 1,
            state = $6,
            last_review = $7
        WHERE id = $8
        "#,
        result.stability,
        result.difficulty,
        due,
        result.interval_days,
        is_lapse,
        state,
        now,
        drill_item_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)] // fields populated by sqlx query
pub struct DueItem {
    pub drill_item_id: i32,
    pub kanji: Option<String>,
    pub kana: String,
    pub meaning: String,
    pub verb_type: String,
    pub form: String,
    pub formality: String,
    pub stability: Option<f32>,
    pub difficulty: Option<f32>,
    pub last_review: Option<DateTime<Utc>>,
    pub reps: i32,
}

pub async fn fetch_due_items(
    pool: &PgPool,
    deck_id: Option<i32>,
    forms: &[String],
    formalities: &[String],
    limit: i64,
) -> sqlx::Result<Vec<DueItem>> {
    sqlx::query_as!(
        DueItem,
        r#"
        SELECT di.id as drill_item_id, v.kanji, v.kana, v.meaning,
               v.verb_type, di.form, di.formality,
               di.stability, di.difficulty, di.last_review, di.reps
        FROM drill_items di
        JOIN verbs v ON v.id = di.verb_id
        WHERE ($1::int IS NULL OR v.deck_id = $1)
          AND di.form = ANY($2)
          AND di.formality = ANY($3)
          AND di.due <= now()
        ORDER BY di.due ASC
        LIMIT $4
        "#,
        deck_id,
        forms,
        formalities,
        limit,
    )
    .fetch_all(pool)
    .await
}
