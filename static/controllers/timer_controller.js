import { Controller } from 'https://cdn.jsdelivr.net/npm/@hotwired/stimulus@3/+esm';

export default class extends Controller {
    static targets = ["timestamp"];

    connect() {
        if (this.hasTimestampTarget) {
            this.timestampTarget.value = Date.now().toString();
        }
    }
}
