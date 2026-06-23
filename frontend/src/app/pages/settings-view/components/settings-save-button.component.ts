import {Component, input} from '@angular/core';
import { MatButton } from '@angular/material/button';
import { MatIcon } from '@angular/material/icon';
import {MatTooltip} from "@angular/material/tooltip";

@Component({
    selector: 'app-settings-save-button',
    imports: [MatButton, MatIcon, MatTooltip],
    template: `
        <button [disabled]="disabled()" type="submit" form="settings-form" mat-flat-button matTooltip="Save All">
            <mat-icon>save</mat-icon>
        </button>
    `,
    })
export class SettingsSaveButtonComponent {
    public readonly disabled = input.required<boolean>();
}






