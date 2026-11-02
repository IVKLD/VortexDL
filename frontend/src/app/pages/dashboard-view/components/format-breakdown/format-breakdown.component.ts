import { Component, input } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { MatProgressBar } from '@angular/material/progress-bar';
import { EmptyPaneComponent } from '@shared/components/empty-pane/empty-pane';
import { FormatItem } from './format-breakdown.model';

@Component({
    selector: 'app-format-breakdown',
    imports: [
        MatProgressBar,
        MatIcon,
        EmptyPaneComponent,
    ],
    templateUrl: './format-breakdown.component.html',
    styleUrl: './format-breakdown.component.scss',
})
export class FormatBreakdownComponent {
    public readonly data = input.required<FormatItem[]>();
}
