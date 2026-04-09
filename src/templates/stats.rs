use maud::{Markup, html};

use crate::{
    db::{
        decks::DeckWithStats,
        sessions::{DrillSession, FormAccuracy, WeakVerb},
    },
    templates::layout,
};

pub fn render(
    decks: &[DeckWithStats],
    form_accuracy: &[FormAccuracy],
    weak_verbs: &[WeakVerb],
    recent_sessions: &[DrillSession],
) -> Markup {
    layout::page(
        "Stats",
        html! {
            section.dashboard {
                h1 { "Stats" }

                @if decks.is_empty() && form_accuracy.is_empty() {
                    p.placeholder { "No data yet. Sync from Anki and start drilling." }
                }

                @if !decks.is_empty() {
                    section.stats-section {
                        h2 { "Decks" }
                        @for deck in decks {
                            div.deck-card {
                                h3 { (deck.anki_name) }
                                div.deck-meta {
                                    span { (deck.verb_count) " verbs" }
                                    span.due { (deck.due_count) " due" }
                                    span { (deck.new_count) " new" }
                                    span { (deck.learning_count) " learning" }
                                    span { (deck.review_count) " review" }
                                }
                            }
                        }
                    }
                }

                @if !form_accuracy.is_empty() {
                    section.stats-section {
                        h2 { "Accuracy by Form" }
                        table.stats-table {
                            thead {
                                tr {
                                    th { "Form" }
                                    th { "Total" }
                                    th { "Correct" }
                                    th { "Rate" }
                                }
                            }
                            tbody {
                                @for fa in form_accuracy {
                                    @let rate = if fa.total > 0 { fa.correct as f64 / fa.total as f64 * 100.0 } else { 0.0 };
                                    tr {
                                        td { (fa.form) }
                                        td { (fa.total) }
                                        td { (fa.correct) }
                                        td {
                                            span.accuracy-bar style=(format!("width: {}px", (rate * 0.6) as u32)) {}
                                            (format!("{:.0}%", rate))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                @if !weak_verbs.is_empty() {
                    section.stats-section {
                        h2 { "Weakest Verbs" }
                        table.stats-table {
                            thead {
                                tr {
                                    th { "Verb" }
                                    th { "Meaning" }
                                    th { "Attempts" }
                                    th { "Rate" }
                                }
                            }
                            tbody {
                                @for v in weak_verbs {
                                    @let rate = if v.total > 0 { v.correct as f64 / v.total as f64 * 100.0 } else { 0.0 };
                                    tr {
                                        td lang="ja" {
                                            (v.kanji.as_deref().unwrap_or(&v.kana))
                                        }
                                        td { (v.meaning) }
                                        td { (v.total) }
                                        td {
                                            span.accuracy-bar style=(format!("width: {}px", (rate * 0.6) as u32)) {}
                                            (format!("{:.0}%", rate))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                @if !recent_sessions.is_empty() {
                    section.stats-section {
                        h2 { "Recent Sessions" }
                        table.stats-table {
                            thead {
                                tr {
                                    th { "Date" }
                                    th { "Correct" }
                                    th { "Wrong" }
                                    th { "Rate" }
                                }
                            }
                            tbody {
                                @for s in recent_sessions {
                                    @let total = s.correct_count + s.wrong_count;
                                    @let rate = if total > 0 { s.correct_count as f64 / total as f64 * 100.0 } else { 0.0 };
                                    tr {
                                        td { (s.started_at.format("%Y-%m-%d %H:%M")) }
                                        td { (s.correct_count) }
                                        td { (s.wrong_count) }
                                        td { (format!("{:.0}%", rate)) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}
