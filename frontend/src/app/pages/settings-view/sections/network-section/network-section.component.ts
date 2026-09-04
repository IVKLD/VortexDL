import { Component, inject, input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatButton } from '@angular/material/button';
import { MatInput } from '@angular/material/input';
import { MatFormField, MatLabel, MatError } from '@angular/material/form-field';
import { FieldTree, FormField } from '@angular/forms/signals';
import { SettingsFormModel } from '../../models/settings-form.model';
import { SettingsSwitchCardComponent } from '../../components/settings-switch-card/settings-switch-card.component';
import { FallbackProxiesComponent } from './components/fallback-proxies/fallback-proxies.component';
import { NetworkState } from './network.state';

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
        FallbackProxiesComponent,
    ],
    providers: [NetworkState],
    templateUrl: './network-section.component.html',
    styleUrl: './network-section.component.scss',
})
export class NetworkSectionComponent {
    protected readonly state = inject(NetworkState);

    public readonly form = input.required<FieldTree<SettingsFormModel['network']>>();
}
