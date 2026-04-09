# Conjanki: Japanese Verb Conjugation Driller

## Context

A Rust web app to drill Japanese verb conjugations from Anki cards the user already knows. Connects to local AnkiConnect, identifies verbs via lindera morphological analysis, generates conjugation drills with jp_inflections, and schedules reviews with FSRS.

## Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, Axum 0.8, tokio |
| Database | PostgreSQL, sqlx 0.8 (with embedded migrations) |
| Config | dotenvy + envy (env), TOML (deck configs) |
| HTTP client | reqwest (AnkiConnect) |
| Templates | Maud 0.27 (SSR) |
| Frontend JS | Turbo Drive + Turbo Frames + Stimulus (CDN, no build) |
| Static files | tower_http::ServeDir |
| Japanese NLP | lindera 3 (IPADIC, embedded dictionary) |
| Conjugation | jp_inflections 0.1 |
| SRS | fsrs (fsrs-rs) |

## Database Schema

### `decks`
```sql
CREATE TABLE decks (
    id          SERIAL PRIMARY KEY,
    anki_name   TEXT NOT NULL UNIQUE,       -- matches [[deck]].name in conjanki.toml
    last_synced TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### `verbs`
```sql
CREATE TABLE verbs (
    id              SERIAL PRIMARY KEY,
    deck_id         INT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    anki_card_id    BIGINT NOT NULL,
    kanji           TEXT,                   -- nullable for kana-only verbs
    kana            TEXT NOT NULL,
    meaning         TEXT NOT NULL,
    verb_type       TEXT NOT NULL,           -- "godan" | "ichidan" | "exception"
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(deck_id, anki_card_id)
);
CREATE INDEX idx_verbs_deck_id ON verbs(deck_id);
```

### `drill_items` (one per verb x form x formality, each with FSRS state)
```sql
CREATE TABLE drill_items (
    id              SERIAL PRIMARY KEY,
    verb_id         INT NOT NULL REFERENCES verbs(id) ON DELETE CASCADE,
    form            TEXT NOT NULL,           -- "negative", "te_form", "past", etc.
    formality       TEXT NOT NULL,           -- "plain" | "polite"
    stability       REAL,                    -- FSRS: NULL for new items
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
```

### `drill_sessions`
```sql
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
```

### `review_log`
```sql
CREATE TABLE review_log (
    id              SERIAL PRIMARY KEY,
    session_id      UUID NOT NULL REFERENCES drill_sessions(id) ON DELETE CASCADE,
    drill_item_id   INT NOT NULL REFERENCES drill_items(id) ON DELETE CASCADE,
    user_answer     TEXT NOT NULL,
    correct_answer  TEXT NOT NULL,           -- JSON array of accepted answers
    is_correct      BOOLEAN NOT NULL,
    rating          SMALLINT NOT NULL,       -- 1=Again, 2=Hard, 3=Good, 4=Easy
    response_ms     INT,
    reviewed_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_review_log_session ON review_log(session_id);
CREATE INDEX idx_review_log_drill_item ON review_log(drill_item_id);
```

## Deck Configuration (`conjanki.toml`)

Per-deck Anki query patterns and field mappings are defined in a TOML config file, not hardcoded. This makes it easy to add new deck types without code changes.

```toml
[[deck]]
name = "Kaishi 1.5k"
query = 'deck:"Kaishi 1.5k" is:review'

[deck.fields]
word = "Word"
reading = "Word Reading"
meaning = "Word Meaning"

[[deck]]
name = "Genki Vocab"
query = 'deck:"Genki 3rd Edition"'

[deck.fields]
word = "UKanji"
reading = "Japanese"
meaning = "English"
```

All verb detection uses lindera — tokenize the `word` field, check POS = verb, extract verb type from conjugation class. This is uniform across all deck types and doesn't require per-deck detection mode configuration. Cards where the `word` field doesn't tokenize as a verb are simply skipped.

### Rust types

```rust
// In src/config.rs, loaded alongside the env-based Config

#[derive(Debug, Deserialize)]
pub struct DeckConfig {
    pub name: String,
    pub query: String,
    pub fields: FieldMapping,
}

#[derive(Debug, Deserialize)]
pub struct FieldMapping {
    pub word: String,
    pub reading: String,
    pub meaning: String,
}

#[derive(Debug, Deserialize)]
pub struct DeckConfigs {
    pub deck: Vec<DeckConfig>,
}
```

Loading: `toml::from_str::<DeckConfigs>(&std::fs::read_to_string("conjanki.toml")?)?`

The sync pipeline iterates over `deck_configs.deck` instead of hardcoded deck types. Each deck config drives which AnkiConnect query to run, which fields to extract, and how to detect verbs.

## Module Structure

```
conjanki/
├── Cargo.toml
├── .env
├── conjanki.toml                 # deck query patterns + field mappings
├── migrations/
│   ├── 20260407000001_initial_schema.up.sql
│   └── 20260407000001_initial_schema.down.sql
├── static/
│   ├── style.css
│   └── controllers/
│       ├── drill_controller.js
│       └── timer_controller.js
└── src/
    ├── main.rs
    ├── config.rs
    ├── error.rs
    ├── state.rs
    ├── anki/
    │   ├── mod.rs
    │   ├── client.rs
    │   └── models.rs
    ├── japanese/
    │   ├── mod.rs
    │   ├── analyzer.rs
    │   └── conjugation.rs
    ├── srs/
    │   ├── mod.rs
    │   └── scheduler.rs
    ├── db/
    │   ├── mod.rs
    │   ├── decks.rs
    │   ├── verbs.rs
    │   ├── drill_items.rs
    │   └── sessions.rs
    ├── routes/
    │   ├── mod.rs
    │   ├── dashboard.rs
    │   ├── sync.rs
    │   ├── drill.rs
    │   └── stats.rs
    └── templates/
        ├── mod.rs
        ├── layout.rs
        ├── dashboard.rs
        ├── sync.rs
        ├── drill.rs
        └── stats.rs
```

## Config (`src/config.rs`)

Two config sources:
1. **Env vars** (via dotenvy + envy): DB connection, bind address, AnkiConnect URL, FSRS retention
2. **TOML file** (`conjanki.toml`): Per-deck query patterns, field mappings, detection mode (see above)

```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    #[serde(default = "default_bind")]
    pub bind_address: String,           // default: 127.0.0.1:3000
    #[serde(default = "default_anki_url")]
    pub anki_connect_url: String,       // default: http://127.0.0.1:8765
    #[serde(default = "default_retention")]
    pub desired_retention: f32,         // default: 0.9
}
```

`toml` crate added to Cargo.toml for parsing `conjanki.toml`.

## AnkiConnect Client (`src/anki/client.rs`)

JSON-RPC v6 over HTTP POST to `anki_connect_url`. Key endpoints:
- `deckNames` -> `Vec<String>`
- `findCards(query)` -> `Vec<i64>` (card IDs)
- `cardsInfo(cards)` -> `Vec<CardInfo>` (batched in chunks of 50)
- `getDeckStats(decks)` -> deck statistics

Query patterns are read from `conjanki.toml` (see Deck Configuration above), not hardcoded.

## Verb Detection (`src/japanese/analyzer.rs`)

All decks use lindera for verb detection — uniform pipeline, no per-deck detection mode.

1. Read the configured `word` field from the card
2. Tokenize with lindera (IPADIC)
3. Check token details[0] == "動詞" (verb)
4. Extract verb type from details[4]: "五段" -> Godan, "一段" -> Ichidan, "サ変"/"カ変" -> Exception
5. Get dictionary form from details[6]
6. Use configured `reading` field for kana, `meaning` field for English
7. Cards that don't contain a verb are silently skipped

## Conjugation Engine (`src/japanese/conjugation.rs`)

`ConjugationForm` enum with 25 variants covering all jp_inflections forms:
dictionary, negative, past, negative_past, te_form, negative_te_form, potential, negative_potential, passive, negative_passive, causative, negative_causative, causative_passive, imperative, volitional, ba_conditional, tara_conditional, zu_form, desiderative, and their negatives where applicable.

`Formality` enum: Plain (WordForm::Short) | Polite (WordForm::Long).

Forms that don't take a WordForm parameter (te_form, passive, causative, ba, tara, zu, desiderative, imperative) only have Plain formality. The `supports_formality()` method controls this.

`conjugate(kanji, kana, verb_type, form, formality) -> Vec<String>` returns all accepted answers (both kana and kanji forms).

## FSRS Integration (`src/srs/scheduler.rs`)

- `FSRS::new(Some(&DEFAULT_PARAMETERS))`
- `next_states(Option<MemoryState>, desired_retention, days_elapsed) -> NextStates`
- Auto-rating: correct answer -> Good(3), wrong -> Again(1)
- Optional Hard(2)/Easy(4) buttons on result screen (future iteration)
- After review: update drill_item's stability, difficulty, due, interval, lapses, reps

## Routes

| Method | Path | Description | Turbo |
|--------|------|-------------|-------|
| GET | `/` | Dashboard: deck list, verb counts, due counts | Drive |
| POST | `/sync` | Sync from AnkiConnect -> DB | Drive |
| GET | `/drill/setup` | Form/formality picker checklist | Drive |
| POST | `/drill/start` | Create session, redirect to session page | Drive |
| GET | `/drill/:id` | Session shell page with Turbo Frame | Drive |
| GET | `/drill/:id/card` | Next due card (Turbo Frame content) | Frame |
| POST | `/drill/:id/answer` | Validate answer, FSRS update, result (Turbo Frame) | Frame |
| GET | `/stats` | Accuracy per form, weak verbs, session history | Drive |

## Drill Session Flow

1. **Setup**: User selects deck(s), checks conjugation forms and formalities
2. **Start**: POST creates `drill_sessions` row, redirects to session page
3. **Session page**: Renders shell with `<turbo-frame id="drill-card" src="/drill/:id/card">`
4. **Card frame**: Fetches most-overdue drill_item (`WHERE due <= now() ORDER BY due ASC LIMIT 1`), shows verb + form name + formality, pre-computes correct answers
5. **Answer**: User types in kana/kanji, form POSTs to /answer. Server compares against correct answers, auto-rates, updates FSRS state, logs review, returns result Turbo Frame
6. **Result frame**: Shows correct/wrong + correct answer + "Next" link targeting the Turbo Frame
7. **Complete**: When no more due items, show session summary (total, correct, accuracy)

## Turbo/Stimulus

- **Turbo Drive**: All page navigation (automatic with the library included)
- **Turbo Frames**: Only on drill session page (card swap without full reload)
- **drill_controller.js**: Autofocus input field when new card loads
- **timer_controller.js**: Record timestamp when card appears (for response time tracking)

## CSS

Hand-written `static/style.css`. The design should feel like a focused, opinionated tool — not a generic dashboard. Think: ink on washi paper, not Bootstrap cards.

Design direction:
- **Palette**: Warm off-black background (not pure #000 or generic #1a1a2e). Muted warm tones — think aged paper, sumi ink. Accent color: a single warm hue (burnt orange, or deep vermillion) used sparingly.
- **Typography**: Japanese-optimized. Serif or semi-serif for verb display (e.g., "Shippori Mincho" from Google Fonts, or system "Hiragino Mincho" with "Noto Serif JP" fallback). Sans-serif for UI chrome. Verb prompts should feel calligraphic, not clinical.
- **Layout**: Asymmetric. Not everything centered. Drill card can sit left-heavy with generous whitespace on the right. Nav is minimal — maybe just bottom or side, not a fat top bar.
- **Feedback**: Correct = subtle (a soft glow or underline shift, not a screaming green box). Wrong = a respectful nudge (warm red, not alarm-bell crimson). No confetti, no emojis.
- **Input**: The text field should feel like writing — large, clean, maybe a bottom-border-only style (no full box). Focus state is visible but not garish.
- **Texture**: Consider a very subtle noise/grain on the background (CSS only, no images). Gives it physicality.
- **Responsive**: Single column. Works on narrow screens. The drill card is the hero element — everything else recedes.

## Stats Page

- **Deck overview**: Per-deck verb count, drill items by state (new/learning/review), due count
- **Form accuracy**: Per-conjugation-form correct rate (from review_log)
- **Weak verbs**: Top 10 verbs by error rate or lapses
- **Session history**: Recent sessions with date, count, accuracy

## Implementation Order

1. **Skeleton**: cargo init, deps, config, error, state, layout, placeholder route, CSS, main.rs
2. **DB + Migrations**: schema SQL, db/ CRUD modules
3. **AnkiConnect client**: reqwest client, test with live Anki
4. **Verb detection**: lindera analyzer, unit tests
5. **Sync pipeline**: POST /sync, dashboard sync button
6. **Conjugation engine**: ConjugationForm, conjugate(), unit tests
7. **FSRS scheduling**: scheduler module, unit tests
8. **Drill UI**: all drill routes + templates + Stimulus controllers
9. **Dashboard + Stats**: real data, stats queries
10. **Polish**: CSS, keyboard shortcuts, edge cases

## Verification

1. `cargo build` compiles
2. `sqlx migrate run` creates schema
3. With Anki running: "Sync from Anki" populates verbs + drill_items
4. Full drill loop: setup -> start -> answer cards -> FSRS updates -> next card -> session complete
5. Stats page shows form accuracy and weak verbs from review data

### Browser testing with rodney

Use `rodney` (headless Chrome CLI) to verify the full user flow after each major phase:

```bash
rodney start --show
rodney open http://127.0.0.1:3000

# Dashboard loads
rodney wait "main"
rodney text "h1"                        # should show dashboard title

# Sync
rodney click "button:has-text('Sync')"  # or appropriate selector
rodney waitstable
rodney text ".sync-result"              # check verb count

# Drill setup
rodney open http://127.0.0.1:3000/drill/setup
rodney wait "form"
rodney click "input[value='negative']"  # check a form
rodney click "input[value='past']"
rodney submit "form"
rodney waitstable

# Drill interaction
rodney wait "#drill-card"
rodney text ".prompt"                   # verb + form displayed
rodney input ".answer-input" "たべない"
rodney submit "form"
rodney waitstable
rodney text ".feedback"                 # "Correct!" or "Incorrect"

# Stats
rodney open http://127.0.0.1:3000/stats
rodney wait "main"
rodney text ".form-accuracy"

rodney stop
```
