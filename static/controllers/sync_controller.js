import { Controller } from 'https://cdn.jsdelivr.net/npm/@hotwired/stimulus@3/+esm';

export default class extends Controller {
    static targets = ["status", "button"];

    async sync(event) {
        event.preventDefault();
        this.buttonTarget.disabled = true;
        this.statusTarget.hidden = false;

        try {
            // Step 1: Anki syncs with AnkiWeb
            this.statusTarget.textContent = "Syncing Anki with AnkiWeb\u2026";
            const ankiResp = await fetch("/sync/anki", { method: "POST" });
            if (!ankiResp.ok) {
                this.statusTarget.textContent = "Anki sync failed: " + ankiResp.statusText;
                this.buttonTarget.disabled = false;
                return;
            }

            // Step 2: fetch fresh data from local AnkiConnect
            this.statusTarget.textContent = "Getting fresh data from AnkiConnect\u2026";
            const dataResp = await fetch("/sync/data", {
                method: "POST",
                headers: { "Accept": "text/html", "Turbo-Frame": "dashboard-content" },
            });
            if (!dataResp.ok) {
                this.statusTarget.textContent = "Data sync failed: " + dataResp.statusText;
                this.buttonTarget.disabled = false;
                return;
            }

            // Replace the turbo-frame content with the response
            const html = await dataResp.text();
            const frame = document.getElementById("dashboard-content");
            if (frame) {
                const tmp = document.createElement("template");
                tmp.innerHTML = html;
                const newFrame = tmp.content.querySelector("turbo-frame#dashboard-content");
                frame.innerHTML = newFrame ? newFrame.innerHTML : html;
            }
        } catch (err) {
            this.statusTarget.textContent = "Sync error: " + err.message;
            this.buttonTarget.disabled = false;
        }
    }
}
