import {ChangeDetectionStrategy, Component, input} from '@angular/core';
import {MatIcon} from '@angular/material/icon';

@Component({
    selector: 'app-stat-card',
    imports: [MatIcon],
    templateUrl: './stat-card.component.html',
    styleUrl: './stat-card.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StatCardComponent {
    icon = input.required<string>();
    label = input.required<string>();
    value = input.required<string | number>();
    iconClass = input<string>('');
}
