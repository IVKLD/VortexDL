import { Component, input, output } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { FieldTree, FormField } from '@angular/forms/signals';
import { MatInput } from "@angular/material/input";
import { MatError, MatFormField, MatHint, MatLabel, MatSuffix } from "@angular/material/form-field";
import { MatSlideToggle } from "@angular/material/slide-toggle";
import { MatButton } from "@angular/material/button";
import { SoundcloudSettings } from "@app/pages/settings-view/models/settings-form.model";

@Component({
    selector: 'app-soundcloud-section',
    imports: [MatIcon, FormField, MatLabel, MatFormField, MatHint, MatError, MatSlideToggle, MatInput, MatButton, MatSuffix],
    templateUrl: './soundcloud-section.component.html',
    styleUrl: './soundcloud-section.component.scss'
})
export class SoundcloudSectionComponent {
    public readonly form = input.required<FieldTree<SoundcloudSettings>>();
    public readonly isTesting = input<boolean>(false);
    public readonly test = output<void>();
}
