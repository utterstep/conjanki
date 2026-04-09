use sqlx::PgPool;

pub async fn upsert(pool: &PgPool, anki_name: &str) -> sqlx::Result<i32> {
    let row = sqlx::query_scalar!(
        r#"
        INSERT INTO decks (anki_name)
        VALUES ($1)
        ON CONFLICT (anki_name) DO UPDATE SET anki_name = EXCLUDED.anki_name
        RETURNING id
        "#,
        anki_name,
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn set_last_synced(pool: &PgPool, deck_id: i32) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE decks SET last_synced = now() WHERE id = $1",
        deck_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)] // fields populated by sqlx query
pub struct DeckWithStats {
    pub id: i32,
    pub anki_name: String,
    pub last_synced: Option<chrono::DateTime<chrono::Utc>>,
    pub verb_count: i64,
    pub due_count: i64,
    pub new_count: i64,
    pub learning_count: i64,
    pub review_count: i64,
}

pub async fn list_with_stats(pool: &PgPool) -> sqlx::Result<Vec<DeckWithStats>> {
    let rows = sqlx::query_as!(
        DeckWithStats,
        r#"
        SELECT
            d.id,
            d.anki_name,
            d.last_synced,
            COALESCE(COUNT(DISTINCT v.id), 0) as "verb_count!",
            COALESCE(SUM(CASE WHEN di.due <= now() AND di.state != 'new' THEN 1 ELSE 0 END), 0) as "due_count!",
            COALESCE(SUM(CASE WHEN di.state = 'new' THEN 1 ELSE 0 END), 0) as "new_count!",
            COALESCE(SUM(CASE WHEN di.state = 'learning' THEN 1 ELSE 0 END), 0) as "learning_count!",
            COALESCE(SUM(CASE WHEN di.state = 'review' THEN 1 ELSE 0 END), 0) as "review_count!"
        FROM decks d
        LEFT JOIN verbs v ON v.deck_id = d.id
        LEFT JOIN drill_items di ON di.verb_id = v.id
        GROUP BY d.id
        ORDER BY d.anki_name
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
