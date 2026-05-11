import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { MatIcon } from '@angular/material/icon';

export interface ActivityDay {
    label: string;
    count: number;
    heightPercent: number;
}

@Component({
    selector: 'app-activity-chart',
    imports: [MatIcon],
    templateUrl: './activity-chart.component.html',
    styleUrl: './activity-chart.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ActivityChartComponent {
    data = input.required<ActivityDay[]>();
}
