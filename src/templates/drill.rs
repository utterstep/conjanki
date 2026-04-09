use maud::{Markup, PreEscaped, html};
use uuid::Uuid;

use crate::{
    db::{decks::DeckWithStats, sessions::DrillSession},
    japanese::conjugation::{ConjugationForm, Formality},
    srs::scheduler::DueItem,
    templates::layout,
};

pub fn setup(decks: &[DeckWithStats]) -> Markup {
    layout::page(
        "Drill Setup",
        html! {
            section.dashboard {
                h1 { "Drill Setup" }

                form.setup-form method="post" action="/drill/start" {
                    div.form-group {
                        h2 { "Deck" }
                        select name="deck_id" {
                            option value="" { "All decks" }
                            @for deck in decks {
                                option value=(deck.id) { (deck.anki_name) " (" (deck.verb_count) " verbs)" }
                            }
                        }
                    }

                    div.form-group {
                        h2 { "Conjugation Forms" }
                        @for (group_name, forms) in ConjugationForm::grouped() {
                            h3.form-group-label { (group_name) }
                            div.checkbox-grid {
                                @for form in *forms {
                                    label {
                                        input type="checkbox" name="forms" value=(form.db_name());
                                        (form.display_name())
                                    }
                                }
                            }
                        }
                    }

                    div.form-group {
                        h2 { "Formality" }
                        div.checkbox-grid {
                            label {
                                input type="checkbox" name="formalities" value="plain" checked;
                                "Plain (short)"
                            }
                            label {
                                input type="checkbox" name="formalities" value="polite" checked;
                                "Polite (long)"
                            }
                        }
                    }

                    button type="submit" .start-btn { "Start Drilling" }
                }
            }
        },
    )
}

pub fn session_page(session: &DrillSession) -> Markup {
    layout::page(
        "Drill",
        html! {
            section.drill-session {
                div.session-header {
                    h1 { "Drill" }
                    div.progress-count {
                        span id="done-count" { (session.correct_count + session.wrong_count) }
                        @if session.total_cards > 0 {
                            " / " (session.total_cards)
                        }
                    }
                }

                turbo-frame id="drill-card" src=(format!("/drill/{}/card", session.id)) {
                    p.placeholder { "Loading..." }
                }
            }
        },
    )
}

pub fn card_frame(
    item: &DueItem,
    session_id: &Uuid,
    form: ConjugationForm,
    formality: Formality,
    answers: &[String],
) -> Markup {
    let answers_json = serde_json::to_string(answers).unwrap_or_default();
    html! {
        turbo-frame id="drill-card" {
            div.drill-card {
                div.prompt {
                    span.verb lang="ja" {
                        (item.kanji.as_deref().unwrap_or(&item.kana))
                    }
                    div.target-form {
                        span.arrow { "\u{2192}" }
                        " "
                        span.form-name { (form.short_name()) }
                        @if form.supports_formality() {
                            span.formality {
                                ", " @match formality {
                                    Formality::Plain => { "plain" },
                                    Formality::Polite => { "polite" },
                                }
                            }
                        }
                    }
                }
                p.meaning.spoiler { (item.meaning) }

                form method="post" action=(format!("/drill/{}/answer", session_id)) {
                    input type="hidden" name="drill_item_id" value=(item.drill_item_id);
                    input type="hidden" name="correct_answers_json" value=(answers_json);
                    input type="hidden" name="started_at_ms" value="0"
                          id="started-at-ms";
                    input.answer-input type="text" name="user_answer"
                          lang="ja" autocomplete="off" autofocus
                          placeholder="conjugated form...";
                    button type="submit" { "Check" }
                }
                script {
                    (PreEscaped(r#"
                        (function() {
                            var el = document.getElementById('started-at-ms');
                            if (el) el.value = Date.now();
                        })();
                    "#))
                }
            }
        }
    }
}

pub struct AnswerContext<'a> {
    pub is_correct: bool,
    pub user_answer: &'a str,
    pub correct_answers: &'a [String],
    pub session_id: &'a Uuid,
    pub meaning: &'a str,
    pub task_display: &'a str,
    pub verb_kana: &'a str,
    pub stem: Option<&'a str>,
    pub explanation: Option<&'a str>,
}

pub fn answer_result_frame(ctx: &AnswerContext) -> Markup {
    html! {
        turbo-frame id="drill-card" {
            @if ctx.is_correct {
                div.result.correct {
                    div.verdict-icon { "○" }
                    p.feedback { "Correct" }
                    p.task-reminder title=(ctx.verb_kana) { (ctx.task_display) }
                    p.correct-answer lang="ja" {
                        (ctx.correct_answers.join(" / "))
                        " "
                        button.speak-btn type="button" onclick="window.__speak()" title="Listen" { "\u{1F50A}" }
                    }
                    a.next-link href=(format!("/drill/{}/card", ctx.session_id))
                      data-turbo-frame="drill-card" autofocus {
                        "Next \u{2192}"
                    }
                }
            } @else {
                div.result.wrong {
                    div.verdict-icon { "✕" }
                    p.feedback { "Incorrect" }
                    p.task-reminder title=(ctx.verb_kana) { (ctx.task_display) }
                    p.user-answer lang="ja" { (ctx.user_answer) }
                    p.correct-answer lang="ja" {
                        (ctx.correct_answers.join(" / "))
                        " "
                        button.speak-btn type="button" onclick="window.__speak()" title="Listen" { "\u{1F50A}" }
                    }
                    p.meaning { (ctx.meaning) }
                    @if ctx.stem.is_some() || ctx.explanation.is_some() {
                        p.conjugation-hint {
                            @if let Some(stem) = ctx.stem {
                                span lang="ja" { "stem: " span.stem-value { (stem) } }
                            }
                            @if ctx.stem.is_some() && ctx.explanation.is_some() {
                                " · "
                            }
                            @if let Some(explanation) = ctx.explanation {
                                span.explanation { (explanation) }
                            }
                        }
                    }
                    a.next-link href=(format!("/drill/{}/card", ctx.session_id))
                      data-turbo-frame="drill-card" autofocus {
                        "Next \u{2192}"
                    }
                }
            }
            // The first correct answer (kana) is used for TTS
            @let tts_text = ctx.correct_answers.first().map(|s| s.as_str()).unwrap_or("");
            script {
                (PreEscaped(format!(r#"
                    (function() {{
                        var el = document.getElementById('done-count');
                        if (el) el.textContent = parseInt(el.textContent || '0') + 1;

                        function pickVoice() {{
                            var voices = speechSynthesis.getVoices();
                            // Prefer enhanced/premium Japanese voices (Kyoko, O-Ren, etc.)
                            var ja = voices.filter(function(v) {{ return v.lang.startsWith('ja'); }});
                            var premium = ja.filter(function(v) {{
                                return /premium|enhanced/i.test(v.name) || v.localService === false;
                            }});
                            return premium[0] || ja[0] || null;
                        }}

                        window.__speak = function() {{
                            speechSynthesis.cancel();
                            var u = new SpeechSynthesisUtterance({text});
                            u.lang = 'ja-JP';
                            u.rate = 0.85;
                            var v = pickVoice();
                            if (v) u.voice = v;
                            speechSynthesis.speak(u);
                        }};

                        if (speechSynthesis.getVoices().length) {{
                            window.__speak();
                        }} else {{
                            speechSynthesis.onvoiceschanged = function() {{ window.__speak(); }};
                        }}
                    }})();
                "#, text = serde_json::to_string(tts_text).unwrap_or_default())))
            }
        }
    }
}

pub fn session_complete(total: i32, correct: i32, wrong: i32) -> Markup {
    let accuracy = if total > 0 {
        (correct as f64 / total as f64 * 100.0) as u32
    } else {
        0
    };

    html! {
        turbo-frame id="drill-card" {
            div.session-complete {
                h2 { "Session Complete" }
                div.session-stats {
                    div.stat-item {
                        div.stat-value { (total) }
                        div.stat-label { "Total" }
                    }
                    div.stat-item {
                        div.stat-value { (correct) }
                        div.stat-label { "Correct" }
                    }
                    div.stat-item {
                        div.stat-value { (wrong) }
                        div.stat-label { "Wrong" }
                    }
                    div.stat-item {
                        div.stat-value { (accuracy) "%" }
                        div.stat-label { "Accuracy" }
                    }
                }
                div.actions {
                    a href="/drill/setup" .start-btn { "Drill Again" }
                    a href="/" .sync-btn { "Dashboard" }
                }
            }
        }
    }
}
