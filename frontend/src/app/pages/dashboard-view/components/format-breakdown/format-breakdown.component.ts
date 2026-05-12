import {ChangeDetectionStrategy, Component, input} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {FormatItem} from '../../dashboard-view.model';

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
