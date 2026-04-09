use maud::{Markup, html};

use crate::{db::decks::DeckWithStats, templates::layout};

pub fn render(decks: &[DeckWithStats], flash: Option<&str>) -> Markup {
    layout::page(
        "Dashboard",
        html! {
            section.dashboard {
                h1 { "conjanki" }
                p.subtitle { "verb conjugation drills" }

                turbo-frame id="dashboard-content" {
                    (dashboard_content(decks, flash))
                }
            }
        },
    )
}

pub fn dashboard_content(decks: &[DeckWithStats], flash: Option<&str>) -> Markup {
    html! {
        @if let Some(msg) = flash {
            div.sync-result { p { (msg) } }
        }

        div data-controller="sync" {
            div.actions {
                button type="button" .sync-btn
                       data-sync-target="button"
                       data-action="click->sync#sync" { "Sync from Anki" }
                a href="/drill/setup" .start-btn data-turbo-frame="_top" { "Start Drilling" }
            }
            p.sync-status hidden data-sync-target="status" {}
        }

        section.deck-list {
            h2 { "Decks" }
            @if decks.is_empty() {
                p.placeholder { "No decks synced yet. Click \"Sync from Anki\" to get started." }
            } @else {
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
    }
}
