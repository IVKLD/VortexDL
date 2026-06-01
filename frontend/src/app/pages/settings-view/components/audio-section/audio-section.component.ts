import {ChangeDetectionStrategy, Component, input} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {FormField} from '@angular/forms/signals';
import {MatFormField, MatLabel} from "@angular/material/input";
import {MatSlideToggle} from "@angular/material/slide-toggle";
import {MatOption, MatSelect} from "@angular/material/select";

@Component({
    selector: 'app-audio-settings',
    imports: [MatIcon, FormField, MatLabel, MatFormField, MatSlideToggle, MatSelect, MatOption],
    templateUrl: './audio-section.component.html',
    styleUrl: './audio-section.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AudioSettingsComponent {
    public readonly form = input.required<any>();
}
