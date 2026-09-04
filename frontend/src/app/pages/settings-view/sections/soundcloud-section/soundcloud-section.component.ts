import { Component, inject, input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatButton } from '@angular/material/button';
import { MatInput } from '@angular/material/input';
import { MatError, MatFormField, MatHint, MatLabel } from '@angular/material/form-field';
import { FieldTree, FormField } from '@angular/forms/signals';
import { SettingsFormModel } from '../../models/settings-form.model';
import { SettingsSwitchCardComponent } from '../../components/settings-switch-card/settings-switch-card.component';
import { MatTooltip } from '@angular/material/tooltip';
import { SoundcloudState } from './soundcloud.state';

@Component({
    selector: 'app-soundcloud-section',
    imports: [
        CommonModule,
        MatIcon,
        MatInput,
        MatFormField,
        MatHint,
        MatLabel,
        MatError,
        FormField,
        SettingsSwitchCardComponent,
        MatTooltip,
        MatButton,
    ],
    providers: [SoundcloudState],
    templateUrl: './soundcloud-section.component.html',
    styleUrl: './soundcloud-section.component.scss',
})
export class SoundcloudSectionComponent {
    protected readonly state = inject(SoundcloudState);

    public readonly form = input.required<FieldTree<SettingsFormModel['soundcloud']>>();
}
