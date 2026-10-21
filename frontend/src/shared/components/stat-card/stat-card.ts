import { Component, input } from '@angular/core';

@Component({
    selector: 'app-stat-card',
    templateUrl: './stat-card.html',
    styleUrl: './stat-card.scss',
})
export class StatCardComponent {
    public readonly label = input.required<string>();
    public readonly value = input.required<string>();
}
