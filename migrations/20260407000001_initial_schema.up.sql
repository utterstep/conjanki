CREATE TABLE decks (
    id          SERIAL PRIMARY KEY,
    anki_name   TEXT NOT NULL UNIQUE,
    last_synced TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE verbs (
    id              SERIAL PRIMARY KEY,
    deck_id         INT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    anki_card_id    BIGINT NOT NULL,
    kanji           TEXT,
    kana            TEXT NOT NULL,
    meaning         TEXT NOT NULL,
    verb_type       TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(deck_id, anki_card_id)
);
CREATE INDEX idx_verbs_deck_id ON verbs(deck_id);

CREATE TABLE drill_items (
    id              SERIAL PRIMARY KEY,
    verb_id         INT NOT NULL REFERENCES verbs(id) ON DELETE CASCADE,
    form            TEXT NOT NULL,
    formality       TEXT NOT NULL,
    stability       REAL,
    difficulty      REAL,
    due             TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_interval   REAL NOT NULL DEFAULT 0,
    lapses          INT NOT NULL DEFAULT 0,
    reps            INT NOT NULL DEFAULT 0,
    state           TEXT NOT NULL DEFAULT 'new',
    last_review     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(verb_id, form, formality)
);
CREATE INDEX idx_drill_items_due ON drill_items(due);

CREATE TABLE drill_sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    deck_id         INT REFERENCES decks(id),
    forms           TEXT[] NOT NULL,
    formalities     TEXT[] NOT NULL,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at     TIMESTAMPTZ,
    total_cards     INT NOT NULL DEFAULT 0,
    correct_count   INT NOT NULL DEFAULT 0,
    wrong_count     INT NOT NULL DEFAULT 0
);

CREATE TABLE review_log (
    id              SERIAL PRIMARY KEY,
    session_id      UUID NOT NULL REFERENCES drill_sessions(id) ON DELETE CASCADE,
    drill_item_id   INT NOT NULL REFERENCES drill_items(id) ON DELETE CASCADE,
    user_answer     TEXT NOT NULL,
    correct_answer  TEXT NOT NULL,
    is_correct      BOOLEAN NOT NULL,
    rating          SMALLINT NOT NULL,
    response_ms     INT,
    reviewed_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_review_log_session ON review_log(session_id);
CREATE INDEX idx_review_log_drill_item ON review_log(drill_item_id);
