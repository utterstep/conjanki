use sqlx::PgPool;

pub async fn upsert(pool: &PgPool, verb_id: i32, form: &str, formality: &str) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO drill_items (verb_id, form, formality)
        VALUES ($1, $2, $3)
        ON CONFLICT (verb_id, form, formality) DO NOTHING
        "#,
        verb_id,
        form,
        formality,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn count_due(
    pool: &PgPool,
    deck_id: Option<i32>,
    forms: &[String],
    formalities: &[String],
) -> sqlx::Result<i64> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM drill_items di
        JOIN verbs v ON v.id = di.verb_id
        WHERE ($1::int IS NULL OR v.deck_id = $1)
          AND di.form = ANY($2)
          AND di.formality = ANY($3)
          AND di.due <= now()
        "#,
        deck_id,
        forms,
        formalities,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

#[derive(Debug)]
#[allow(dead_code)] // fields populated by sqlx query
pub struct DrillItemWithVerb {
    pub id: i32,
    pub form: String,
    pub formality: String,
    pub stability: Option<f32>,
    pub difficulty: Option<f32>,
    pub last_review: Option<chrono::DateTime<chrono::Utc>>,
    pub kanji: Option<String>,
    pub kana: String,
    pub meaning: String,
    pub verb_type: String,
}

pub async fn get_with_verb(pool: &PgPool, id: i32) -> sqlx::Result<DrillItemWithVerb> {
    sqlx::query_as!(
        DrillItemWithVerb,
        r#"
        SELECT di.id, di.form, di.formality, di.stability, di.difficulty, di.last_review,
               v.kanji, v.kana, v.meaning, v.verb_type
        FROM drill_items di
        JOIN verbs v ON v.id = di.verb_id
        WHERE di.id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await
}
