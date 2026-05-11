import {Component, input, output} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {FormField} from '@angular/forms/signals';
import {MatError, MatFormField, MatInput, MatLabel} from "@angular/material/input";
import {MatSlideToggle} from "@angular/material/slide-toggle";
import {MatIconButton} from "@angular/material/button";

@Component({
    selector: 'app-network-settings',
    imports: [MatIcon, FormField, MatLabel, MatFormField, MatError, MatSlideToggle, MatInput, MatIconButton],
    templateUrl: './network-section.component.html',
    styleUrl: './network-section.component.scss'
})
export class NetworkSettingsComponent {
    public readonly form = input.required<any>();
    public readonly isTesting = input<boolean>(false);
    public readonly test = output<void>();
}
