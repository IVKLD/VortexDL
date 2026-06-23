import {Component, input} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {FormatItem} from '../../dashboard-view.model';

@Component({
    selector: 'app-format-breakdown',
    imports: [MatIcon],
    templateUrl: './format-breakdown.component.html',
    styleUrl: './format-breakdown.component.scss',
    })
export class FormatBreakdownComponent {
    public readonly data = input.required<FormatItem[]>();
}
