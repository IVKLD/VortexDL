import { Component, input } from '@angular/core';
import { MatIcon } from '@angular/material/icon';

@Component({
    selector: 'app-empty-pane',
    imports: [MatIcon],
    templateUrl: './empty-pane.html',
    styleUrl: './empty-pane.scss',
})
export class EmptyPaneComponent {
    public icon = input.required<string>();
    public heading = input.required<string>();
    public description = input<string>();
    public accent = input<boolean>(false);
}
