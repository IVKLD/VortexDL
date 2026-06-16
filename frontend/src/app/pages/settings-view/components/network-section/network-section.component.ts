import {ChangeDetectionStrategy, Component, input, output} from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { FormField } from '@angular/forms/signals';
import { MatInput } from "@angular/material/input";
import { MatError, MatFormField, MatLabel, MatSuffix } from "@angular/material/form-field";
import { MatSlideToggle } from "@angular/material/slide-toggle";
import { MatIconButton } from "@angular/material/button";
import { FallbackProxiesComponent } from './components/fallback-proxies/fallback-proxies.component';

@Component({
    selector: 'app-network-settings',
    imports: [
        MatIcon, FormField, MatLabel, MatFormField, MatError, MatSlideToggle, MatInput, MatIconButton, MatSuffix,
        FallbackProxiesComponent
    ],
    templateUrl: './network-section.component.html',
    styleUrl: './network-section.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NetworkSettingsComponent {
    public readonly form = input.required<any>();
    public readonly isTesting = input<boolean>(false);
    public readonly test = output<void>();
}
