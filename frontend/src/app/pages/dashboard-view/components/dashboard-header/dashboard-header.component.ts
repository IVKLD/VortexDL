import {ChangeDetectionStrategy, Component} from '@angular/core';

@Component({
    selector: 'app-dashboard-header',
    templateUrl: './dashboard-header.component.html',
    styleUrl: './dashboard-header.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardHeaderComponent {
}
