import { Component, inject, input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatButton, MatIconButton } from '@angular/material/button';
import { MatButtonToggle, MatButtonToggleGroup } from '@angular/material/button-toggle';
import { MatInput } from '@angular/material/input';
import { MatError, MatFormField, MatHint, MatLabel, MatSuffix } from '@angular/material/form-field';
import { MatTooltip } from '@angular/material/tooltip';
import { FieldTree, FormField } from '@angular/forms/signals';
import { PasswordToggleDirective } from '@shared/directives/password-toggle.directive';
import { SettingsFormModel } from '../../models/settings-form.model';
import { SyncProviderType } from '../../models/sync.model';
import { SyncState } from './sync.state';

@Component({
    selector: 'app-sync-section',
    imports: [
        CommonModule,
        MatIcon,
        MatInput,
        MatFormField,
        MatHint,
        MatLabel,
        MatSuffix,
        MatError,
        FormField,
        MatTooltip,
        MatButton,
        MatIconButton,
        MatButtonToggleGroup,
        MatButtonToggle,
        PasswordToggleDirective,
    ],
    providers: [SyncState],
    templateUrl: './sync-section.component.html',
    styleUrl: './sync-section.component.scss',
})
export class SyncSectionComponent {
    protected readonly syncState = inject(SyncState);

    public readonly form = input.required<FieldTree<SettingsFormModel['webdav']>>();
    public readonly SyncProviderType = SyncProviderType;
}

