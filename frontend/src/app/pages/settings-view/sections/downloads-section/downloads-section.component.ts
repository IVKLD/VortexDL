import { Component, input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatInput } from '@angular/material/input';
import { MatFormField, MatHint, MatLabel, MatError } from '@angular/material/form-field';
import { FieldTree, FormField } from '@angular/forms/signals';
import { SettingsFormModel } from '../../models/settings-form.model';

@Component({
    selector: 'app-downloads-section',
    imports: [
        CommonModule,
        MatIcon,
        MatInput,
        MatFormField,
        MatHint,
        MatLabel,
        MatError,
        FormField
    ],
    templateUrl: './downloads-section.component.html',
    styleUrl: './downloads-section.component.scss'
})
export class DownloadsSectionComponent {
    public readonly downloadsForm = input.required<FieldTree<SettingsFormModel['downloads']>>();
    public readonly systemForm = input.required<FieldTree<SettingsFormModel['system']>>();
}
