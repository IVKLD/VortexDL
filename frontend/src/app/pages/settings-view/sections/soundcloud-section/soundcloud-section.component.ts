import { Component, input, output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatButton } from '@angular/material/button';
import { MatInput } from '@angular/material/input';
import { MatFormField, MatHint, MatLabel, MatError } from '@angular/material/form-field';
import { FieldTree, FormField, ValidationError } from '@angular/forms/signals';
import { SettingsFormModel } from '../../models/settings-form.model';
import { SettingsSwitchCardComponent } from '../../components/settings-switch-card/settings-switch-card.component';

@Component({
    selector: 'app-soundcloud-section',
    imports: [
        CommonModule,
        MatIcon,
        MatButton,
        MatInput,
        MatFormField,
        MatHint,
        MatLabel,
        MatError,
        FormField,
        SettingsSwitchCardComponent
    ],
    templateUrl: './soundcloud-section.component.html',
    styleUrl: './soundcloud-section.component.scss'
})
export class SoundcloudSectionComponent {
    public readonly form = input.required<FieldTree<SettingsFormModel['soundcloud']>>();
    public readonly isTesting = input.required<boolean>();
    public readonly testSoundcloud = output<void>();

    protected hasFormatError(field: FieldTree<string>): boolean {
        return field().errors().some((e: ValidationError) => e.kind !== 'testFailed');
    }
}
