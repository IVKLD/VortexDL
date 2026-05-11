import { Component, input, output } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { FormField, FieldTree } from '@angular/forms/signals';
import { MatError, MatFormField, MatHint, MatInput, MatLabel } from "@angular/material/input";
import { MatSlideToggle } from "@angular/material/slide-toggle";
import { MatButton } from "@angular/material/button";
import { SoundcloudSettings } from "@app/pages/settings/models/settings-form.model";

@Component({
    selector: 'app-soundcloud-section',
    imports: [MatIcon, FormField, MatLabel, MatFormField, MatHint, MatError, MatSlideToggle, MatInput, MatButton],
    templateUrl: './soundcloud-section.component.html',
    styleUrl: './soundcloud-section.component.scss'
})
export class SoundcloudSectionComponent {
    public readonly form = input.required<FieldTree<SoundcloudSettings>>();
    public readonly isTesting = input<boolean>(false);
    public readonly test = output<void>();
}
