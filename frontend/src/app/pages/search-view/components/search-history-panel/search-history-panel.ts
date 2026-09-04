import {Component, inject, output} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {MatIconButton} from '@angular/material/button';
import {MatTooltip} from '@angular/material/tooltip';
import {SearchHistoryService} from '../../search-history.service';

@Component({
    selector: 'app-search-history-panel',
    imports: [MatIcon, MatIconButton, MatTooltip],
    templateUrl: './search-history-panel.html',
    styleUrl: './search-history-panel.scss',
})
export class SearchHistoryPanelComponent {
    protected readonly history = inject(SearchHistoryService);
    protected readonly selectItem = output<string>();
}
