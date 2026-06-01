import {ChangeDetectionStrategy, Component, input} from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { FieldTree, FormField } from '@angular/forms/signals';
import { MatInput } from "@angular/material/input";
import { MatFormField, MatHint, MatLabel, MatPrefix } from "@angular/material/form-field";
import { MatSelect, MatOption } from "@angular/material/select";
import { DownloadSettings } from "@app/pages/settings-view/models/settings-form.model";

@Component({
    selector: 'app-downloads-section',
    imports: [MatIcon, FormField, MatLabel, MatFormField, MatHint, MatInput, MatSelect, MatOption, MatPrefix],
    templateUrl: './downloads-section.component.html',
    styleUrl: './downloads-section.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DownloadsSectionComponent {
    public readonly form = input.required<FieldTree<DownloadSettings>>();
}
