use sqlx::PgPool;

pub async fn upsert(
    pool: &PgPool,
    deck_id: i32,
    anki_card_id: i64,
    kanji: Option<&str>,
    kana: &str,
    meaning: &str,
    verb_type: &str,
) -> sqlx::Result<i32> {
    let row = sqlx::query_scalar!(
        r#"
        INSERT INTO verbs (deck_id, anki_card_id, kanji, kana, meaning, verb_type)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (deck_id, anki_card_id) DO UPDATE SET
            kanji = EXCLUDED.kanji,
            kana = EXCLUDED.kana,
            meaning = EXCLUDED.meaning,
            verb_type = EXCLUDED.verb_type
        RETURNING id
        "#,
        deck_id,
        anki_card_id,
        kanji,
        kana,
        meaning,
        verb_type,
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}
