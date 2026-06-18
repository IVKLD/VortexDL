import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { MatButton } from '@angular/material/button';
import { MatIcon } from '@angular/material/icon';

@Component({
    selector: 'app-settings-save-button',
    imports: [MatButton, MatIcon],
    template: `
        <button [disabled]="disabled()" type="submit" form="settings-form" mat-flat-button>
            <mat-icon>save</mat-icon>
            Save All
        </button>
    `,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class SettingsSaveButtonComponent {
    public readonly disabled = input.required<boolean>();
}






