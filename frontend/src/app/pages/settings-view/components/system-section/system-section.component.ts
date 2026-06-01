import {ChangeDetectionStrategy, Component, input} from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { FieldTree, FormField } from '@angular/forms/signals';
import { MatInput } from "@angular/material/input";
import { MatFormField, MatLabel, MatPrefix } from "@angular/material/form-field";
import { SystemSettings } from "@app/pages/settings-view/models/settings-form.model";

@Component({
    selector: 'app-system-section',
    imports: [MatIcon, FormField, MatLabel, MatFormField, MatInput, MatPrefix],
    templateUrl: './system-section.component.html',
    styleUrl: './system-section.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SystemSectionComponent {
    public readonly form = input.required<FieldTree<SystemSettings>>();
}
