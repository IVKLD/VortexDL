import { Component, input } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { EmptyPaneComponent } from '@shared/components/empty-pane/empty-pane';
import { ActivityDay } from '../../dashboard-view.model';

@Component({
    selector: 'app-activity-chart',
    imports: [MatIcon, EmptyPaneComponent],
    templateUrl: './activity-chart.component.html',
    styleUrl: './activity-chart.component.scss',
})
export class ActivityChartComponent {
    public readonly data = input.required<ActivityDay[]>();
}
