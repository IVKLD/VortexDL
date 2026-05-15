import {ChangeDetectionStrategy, Component, input} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {RouterLink} from '@angular/router';

@Component({
    selector: 'app-section-header',
    imports: [MatIcon, RouterLink],
    templateUrl: './section-header.html',
    styleUrl: './section-header.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SectionHeaderComponent {
    public title = input.required<string>();
    public icon = input.required<string>();
    public link = input<{ label: string; url: string }>();
}
