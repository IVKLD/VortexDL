import { Component, input } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { EmptyPaneComponent } from '@shared/components/empty-pane/empty-pane';
import { FormatItem } from '../../dashboard-view.model';

@Component({
    selector: 'app-format-breakdown',
    imports: [MatIcon, EmptyPaneComponent],
    templateUrl: './format-breakdown.component.html',
    styleUrl: './format-breakdown.component.scss',
})
export class FormatBreakdownComponent {
    public readonly data = input.required<FormatItem[]>();
}
