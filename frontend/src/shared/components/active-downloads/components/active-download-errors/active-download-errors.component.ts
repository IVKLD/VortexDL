import { Component, input, output } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { MatButton } from '@angular/material/button';
import { MatIcon } from '@angular/material/icon';
import { MatDivider } from '@angular/material/list';
import { MatTooltip } from '@angular/material/tooltip';
import { timer } from 'rxjs';

@Component({
    selector: 'app-active-download-errors',
    imports: [MatIcon, MatDivider, MatTooltip, MatButton],
    templateUrl: './active-download-errors.component.html',
    styleUrl: './active-download-errors.component.scss',
})
export class ActiveDownloadErrorsComponent {
    public readonly errors = input.required<string[]>();

    public readonly clear = output<void>();
    public readonly openErrorsDialog = output<void>();
    public readonly autoClose = output<void>();

    constructor() {
        timer(10_000)
            .pipe(takeUntilDestroyed())
            .subscribe(() => this.autoClose.emit());
    }
}
