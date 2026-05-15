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
    public readonly icon = input.required<string>();
    public readonly label = input.required<string>();
    public readonly value = input.required<string | number>();
    public readonly iconClass = input<string>('');
}
