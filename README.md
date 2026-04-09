<p align="center">
  <img src="docs/conjanki_hero.png" alt="conjanki" width="480">
</p>

# conjanki

Japanese verb conjugation driller that pulls vocabulary from your Anki decks via [AnkiConnect](https://foosoft.net/projects/anki-connect/). Verbs are detected automatically with morphological analysis, and conjugation practice is scheduled using [FSRS](https://github.com/open-spaced-repetition/fsrs-rs) spaced repetition.

## How it works

1. **Sync** — connects to your local Anki instance, fetches cards matching configurable queries, identifies verbs via [lindera](https://github.com/lindera/lindera) (IPADIC), and stores them in PostgreSQL.
2. **Drill** — pick conjugation forms (grouped by Genki textbook order) and formality levels, then type conjugated forms. Answers are validated against [jp_inflections](https://crates.io/crates/jp_inflections).
3. **Review** — correct/incorrect ratings feed FSRS to schedule future reviews. Stats track accuracy per form and surface weak verbs.

## Stack

| Layer | Tech |
|-------|------|
| Backend | Rust, Axum, sqlx + PostgreSQL |
| Templates | Maud (compile-time HTML) |
| Frontend | Turbo Drive + Turbo Frames + Stimulus (CDN, no build step) |
| Japanese NLP | lindera (morphological analysis), jp_inflections (conjugation) |
| SRS | fsrs-rs |
| Anki integration | AnkiConnect JSON-RPC via reqwest |

## Prerequisites

- Rust (2024 edition)
- PostgreSQL
- [Anki](https://apps.ankiweb.net/) with [AnkiConnect](https://foosoft.net/projects/anki-connect/) add-on installed and running

## Setup

```bash
# Create the database
createdb conjanki

# Configure environment
cp .env.example .env
# Edit .env with your DATABASE_URL, BIND_ADDRESS, etc.
```

### Environment variables (`.env`)

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | — | PostgreSQL connection string |
| `BIND_ADDRESS` | `0.0.0.0:3000` | Server listen address |
| `ANKI_CONNECT_URL` | `http://127.0.0.1:8765` | AnkiConnect endpoint |
| `DESIRED_RETENTION` | `0.9` | FSRS target retention (0.0–1.0) |

### Deck configuration (`conjanki.toml`)

Map your Anki decks to the fields conjanki needs:

```toml
[[deck]]
name = "Kaishi 1.5k"
query = 'deck:"Kaishi 1.5k" is:review'

[deck.fields]
word = "Word"          # field containing the Japanese word
reading = "Word Reading"  # field with kana reading
meaning = "Word Meaning"  # field with English meaning
```

Add as many `[[deck]]` blocks as you like. The `query` uses Anki's search syntax — `is:review` limits to cards you've already studied.

## Running

```bash
cargo run
```

Open `http://localhost:3000`. Click **Sync from Anki** to import verbs, then **Start Drilling**.

## Conjugation forms

16 forms available, grouped by Genki textbook progression:

- **Basic** (ch. 3–8): dictionary, negative, past, negative past, te-form, negative te-form
- **Intermediate** (ch. 13+): potential, negative potential
- **Advanced** (ch. 21–23): passive, causative, causative passive, and their negatives
- **Other**: imperative, negative imperative

Each form can be drilled in plain (short) and/or polite (long) formality.

## Features

- Syncs Anki with AnkiWeb before fetching local data (two-phase sync with progress)
- Automatic verb detection filters out phrases, conjugated forms, and non-verbs
- Brush-style kanji display (Shippori Mincho) with clean gothic on hover for reading practice
- Meaning hidden behind spoiler until hover
- TTS autoplay of correct answers (Web Speech API) with replay button
- Conjugation hints on incorrect answers: stem, verb type, and formation rule
- Keyboard-driven flow: type answer, Enter to submit, Enter to advance
- Mobile-friendly layout

## License

MIT
