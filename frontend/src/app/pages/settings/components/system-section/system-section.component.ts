import {Component, input} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {FieldTree, FormField} from '@angular/forms/signals';
import {MatFormField, MatInput, MatLabel} from "@angular/material/input";
import {SystemSettings} from "@app/pages/settings/models/settings-form.model";

@Component({
    selector: 'app-system-section',
    imports: [MatIcon, FormField, MatLabel, MatFormField, MatInput],
    templateUrl: './system-section.component.html',
    styleUrl: './system-section.component.scss'
})
export class SystemSectionComponent {
    public readonly form = input.required<FieldTree<SystemSettings>>();
}
