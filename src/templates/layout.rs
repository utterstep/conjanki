use maud::{DOCTYPE, Markup, PreEscaped, html};

pub fn page(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="ja" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " — conjanki" }
                link rel="stylesheet" href="/static/style.css";
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;700&family=Shippori+Mincho:wght@400;600;700&display=swap";
                script type="module" {
                    (PreEscaped(r#"
                        import hotwiredTurbo from 'https://cdn.jsdelivr.net/npm/@hotwired/turbo@8/+esm';
                    "#))
                }
                script type="module" {
                    (PreEscaped(r#"
                        import { Application } from 'https://cdn.jsdelivr.net/npm/@hotwired/stimulus@3/+esm';
                        const app = Application.start();

                        import SyncController from '/static/controllers/sync_controller.js';
                        app.register('sync', SyncController);
                    "#))
                }
            }
            body {
                nav.site-nav {
                    a href="/" { "conjanki" }
                    div.nav-links {
                        a href="/drill/setup" { "Drill" }
                        a href="/stats" { "Stats" }
                    }
                }
                main { (content) }
            }
        }
    }
}
