import { ScrollStrategy } from '@angular/cdk/overlay';

export class CustomBlockScrollStrategy implements ScrollStrategy {
    private _previousHtmlOverflow = '';
    private _previousBodyOverflow = '';
    private _isEnabled = false;

    public attach(): void {}

    public enable(): void {
        if (this._isEnabled) return;
        this._isEnabled = true;

        const html = document.documentElement;
        const body = document.body;

        this._previousHtmlOverflow = html.style.overflow || '';
        this._previousBodyOverflow = body.style.overflow || '';

        html.style.overflow = 'hidden';
        body.style.overflow = 'hidden';
    }

    public disable(): void {
        if (!this._isEnabled) return;
        this._isEnabled = false;

        const html = document.documentElement;
        const body = document.body;

        html.style.overflow = this._previousHtmlOverflow;
        body.style.overflow = this._previousBodyOverflow;
    }
}
