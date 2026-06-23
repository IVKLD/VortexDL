import {Component, input, output} from '@angular/core';
import {MatButton} from '@angular/material/button';
import {MatIcon} from '@angular/material/icon';

@Component({
    selector: 'app-selection-bar',
    imports: [MatButton, MatIcon],
    templateUrl: './selection-bar.html',
    styleUrl: './selection-bar.scss'
})
export class SelectionBar {
    public readonly count = input<number>(0);
    public readonly deleteSelected = output<void>();
    public readonly clearSelection = output<void>();
}
