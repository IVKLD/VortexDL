import { Component, inject, OnInit } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { StatCardComponent } from '@shared/components/stat-card/stat-card';
import { ActivityChartComponent } from './components/activity-chart/activity-chart.component';
import { DeletedTracksComponent } from './components/deleted-tracks/deleted-tracks.component';
import { FormatBreakdownComponent } from './components/format-breakdown/format-breakdown.component';
import { DashboardViewState } from './dashboard-view.state';

@Component({
    selector: 'app-dashboard-view',
    imports: [
        StatCardComponent,
        ActivityChartComponent,
        FormatBreakdownComponent,
        DeletedTracksComponent,
        MatIcon,
    ],
    templateUrl: './dashboard-view.html',
    styleUrl: './dashboard-view.scss',
})
export class DashboardView implements OnInit {
    protected readonly state = inject(DashboardViewState);

    public ngOnInit(): void {
        this.state.loadStats();
    }
}
