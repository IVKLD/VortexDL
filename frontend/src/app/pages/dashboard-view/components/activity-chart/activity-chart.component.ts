import {Component, input} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {ActivityDay} from '../../dashboard-view.model';

@Component({
    selector: 'app-activity-chart',
    imports: [MatIcon],
    templateUrl: './activity-chart.component.html',
    styleUrl: './activity-chart.component.scss',
    })
export class ActivityChartComponent {
    public readonly data = input.required<ActivityDay[]>();
}
