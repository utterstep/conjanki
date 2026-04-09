use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    deck_id: Option<i32>,
    forms: &[String],
    formalities: &[String],
) -> sqlx::Result<Uuid> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO drill_sessions (deck_id, forms, formalities)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        deck_id,
        forms,
        formalities,
    )
    .fetch_one(pool)
    .await?;
    Ok(id)
}

#[derive(Debug)]
#[allow(dead_code)] // fields populated by sqlx query
pub struct DrillSession {
    pub id: Uuid,
    pub deck_id: Option<i32>,
    pub forms: Vec<String>,
    pub formalities: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub total_cards: i32,
    pub correct_count: i32,
    pub wrong_count: i32,
}

pub async fn get(pool: &PgPool, id: Uuid) -> sqlx::Result<DrillSession> {
    sqlx::query_as!(
        DrillSession,
        r#"SELECT id, deck_id, forms, formalities, started_at, finished_at,
                  total_cards, correct_count, wrong_count
           FROM drill_sessions WHERE id = $1"#,
        id,
    )
    .fetch_one(pool)
    .await
}

pub async fn set_total(pool: &PgPool, id: Uuid, total: i32) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE drill_sessions SET total_cards = $1 WHERE id = $2",
        total,
        id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn increment_counts(pool: &PgPool, id: Uuid, correct: bool) -> sqlx::Result<()> {
    if correct {
        sqlx::query!(
            "UPDATE drill_sessions SET correct_count = correct_count + 1 WHERE id = $1",
            id,
        )
        .execute(pool)
        .await?;
    } else {
        sqlx::query!(
            "UPDATE drill_sessions SET wrong_count = wrong_count + 1 WHERE id = $1",
            id,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub struct ReviewEntry<'a> {
    pub session_id: Uuid,
    pub drill_item_id: i32,
    pub user_answer: &'a str,
    pub correct_answers: &'a [String],
    pub is_correct: bool,
    pub rating: i16,
    pub response_ms: Option<i32>,
}

pub async fn log_review(pool: &PgPool, entry: &ReviewEntry<'_>) -> sqlx::Result<()> {
    let correct_json = serde_json::to_string(entry.correct_answers).unwrap_or_default();
    sqlx::query!(
        r#"
        INSERT INTO review_log (session_id, drill_item_id, user_answer, correct_answer,
                                is_correct, rating, response_ms)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        entry.session_id,
        entry.drill_item_id,
        entry.user_answer,
        correct_json,
        entry.is_correct,
        entry.rating,
        entry.response_ms,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug)]
pub struct FormAccuracy {
    pub form: String,
    pub total: i64,
    pub correct: i64,
}

pub async fn accuracy_by_form(pool: &PgPool) -> sqlx::Result<Vec<FormAccuracy>> {
    sqlx::query_as!(
        FormAccuracy,
        r#"
        SELECT di.form,
               COUNT(*) as "total!",
               SUM(CASE WHEN rl.is_correct THEN 1 ELSE 0 END) as "correct!"
        FROM review_log rl
        JOIN drill_items di ON di.id = rl.drill_item_id
        GROUP BY di.form
        ORDER BY di.form
        "#,
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug)]
pub struct WeakVerb {
    pub kanji: Option<String>,
    pub kana: String,
    pub meaning: String,
    pub total: i64,
    pub correct: i64,
}

pub async fn weakest_verbs(pool: &PgPool, limit: i64) -> sqlx::Result<Vec<WeakVerb>> {
    sqlx::query_as!(
        WeakVerb,
        r#"
        SELECT v.kanji, v.kana, v.meaning,
               COUNT(*) as "total!",
               SUM(CASE WHEN rl.is_correct THEN 1 ELSE 0 END) as "correct!"
        FROM review_log rl
        JOIN drill_items di ON di.id = rl.drill_item_id
        JOIN verbs v ON v.id = di.verb_id
        GROUP BY v.id, v.kanji, v.kana, v.meaning
        HAVING COUNT(*) >= 3
        ORDER BY SUM(CASE WHEN rl.is_correct THEN 1 ELSE 0 END)::float / COUNT(*)::float ASC
        LIMIT $1
        "#,
        limit,
    )
    .fetch_all(pool)
    .await
}

pub async fn recent(pool: &PgPool, limit: i64) -> sqlx::Result<Vec<DrillSession>> {
    sqlx::query_as!(
        DrillSession,
        r#"
        SELECT id, deck_id, forms, formalities, started_at, finished_at,
               total_cards, correct_count, wrong_count
        FROM drill_sessions
        ORDER BY started_at DESC
        LIMIT $1
        "#,
        limit,
    )
    .fetch_all(pool)
    .await
}
