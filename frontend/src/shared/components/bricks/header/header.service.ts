import { Injectable, signal, TemplateRef } from '@angular/core';

@Injectable({
    providedIn: 'root'
})
export class HeaderService {
    private readonly _template = signal<TemplateRef<void> | null>(null);
    public readonly template = this._template.asReadonly();

    public setTemplate(template: TemplateRef<void>): void {
        this._template.set(template);
    }

    public clearTemplate(): void {
        this._template.set(null);
    }
}









