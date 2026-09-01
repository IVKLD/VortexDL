import {ScrollStrategy} from '@angular/cdk/overlay';
import {DOCUMENT, inject, Injectable} from "@angular/core";

@Injectable({providedIn: 'root'})
export class StickyAwareScrollStrategy implements ScrollStrategy {
    private document = inject(DOCUMENT);
    private isNestedOpen = false;
    private scrollTop = 0;

    attach(): void {
    }

    enable(): void {
        if (this.isNestedOpen) return;

        this.scrollTop =
            window.pageYOffset || this.document.documentElement.scrollTop;
        const body = this.document.body;

        body.classList.add('scroll-lock-active');
        body.style.overflow = 'hidden';

        this.isNestedOpen = true;
    }

    disable(): void {
        if (!this.isNestedOpen) return;

        const html = this.document.documentElement;
        const body = this.document.body;

        body.classList.remove('scroll-lock-active');
        body.style.overflow = '';
        html.style.removeProperty('--scroll-offset');

        window.scrollTo(0, this.scrollTop);
        this.isNestedOpen = false;
    }
}
