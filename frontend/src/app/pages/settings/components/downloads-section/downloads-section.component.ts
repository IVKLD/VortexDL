import {Component, input} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {FieldTree, FormField} from '@angular/forms/signals';
import {MatFormField, MatHint, MatInput, MatLabel} from "@angular/material/input";
import {DownloadSettings} from "@app/pages/settings/models/settings-form.model";

@Component({
    selector: 'app-downloads-section',
    imports: [MatIcon, FormField, MatLabel, MatFormField, MatHint, MatInput],
    templateUrl: './downloads-section.component.html',
    styleUrl: './downloads-section.component.scss'
})
export class DownloadsSectionComponent {
    public readonly form = input.required<FieldTree<DownloadSettings>>();
}
