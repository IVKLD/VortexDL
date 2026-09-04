import { Directive, signal } from '@angular/core';

@Directive({
    selector: 'input[appPasswordToggle]',
    exportAs: 'passwordToggle',
    host: {
        '[type]': 'hide() ? "password" : "text"',
    },
})
export class PasswordToggleDirective {
    public readonly hide = signal(true);

    public toggle(): void {
        this.hide.update((v) => !v);
    }
}
