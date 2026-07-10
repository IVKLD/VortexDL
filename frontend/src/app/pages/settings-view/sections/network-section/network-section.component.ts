import { Component, input, output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatButton } from '@angular/material/button';
import { MatInput } from '@angular/material/input';
import { MatFormField, MatLabel, MatError } from '@angular/material/form-field';
import { FieldTree, FormField, ValidationError } from '@angular/forms/signals';
import { SettingsFormModel } from '../../models/settings-form.model';
import { SettingsSwitchCardComponent } from '../../components/settings-switch-card/settings-switch-card.component';
import { FallbackProxiesComponent } from './components/fallback-proxies/fallback-proxies.component';

@Component({
    selector: 'app-network-section',
    imports: [
        CommonModule,
        MatIcon,
        MatButton,
        MatInput,
        MatFormField,
        MatLabel,
        MatError,
        FormField,
        SettingsSwitchCardComponent,
        FallbackProxiesComponent
    ],
    templateUrl: './network-section.component.html',
    styleUrl: './network-section.component.scss'
})
export class NetworkSectionComponent {
    public readonly form = input.required<FieldTree<SettingsFormModel['network']>>();
    public readonly isTesting = input.required<boolean>();
    public readonly testProxy = output<void>();

    protected hasFormatError(field: FieldTree<string>): boolean {
        return field().errors().some((e: ValidationError) => e.kind !== 'testFailed');
    }
}
