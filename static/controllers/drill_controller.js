import { Controller } from 'https://cdn.jsdelivr.net/npm/@hotwired/stimulus@3/+esm';

export default class extends Controller {
    static targets = ["input"];

    connect() {
        if (this.hasInputTarget) {
            this.inputTarget.focus();
        }
    }
}
