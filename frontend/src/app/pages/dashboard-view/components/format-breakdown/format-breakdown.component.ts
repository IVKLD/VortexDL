import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { MatIcon } from '@angular/material/icon';

export interface FormatItem {
    format: string;
    count: number;
    percentage: number;
    color: string;
}

@Component({
    selector: 'app-format-breakdown',
    imports: [MatIcon],
    templateUrl: './format-breakdown.component.html',
    styleUrl: './format-breakdown.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class FormatBreakdownComponent {
    data = input.required<FormatItem[]>();
}
