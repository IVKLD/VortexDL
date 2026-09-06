import { Component, inject } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { MatButton } from '@angular/material/button';
import { MatDivider } from '@angular/material/divider';
import { MatMenu, MatMenuItem, MatMenuTrigger } from '@angular/material/menu';
import { SearchViewState } from '../../search-view.state';
import {
    DURATION_OPTIONS,
    SORT_OPTIONS,
    SearchDurationFilter,
    SearchProvider,
    SearchSortOption,
} from '../../models/search-view.model';

@Component({
    selector: 'app-search-toolbar',
    imports: [MatIcon, MatButton, MatMenu, MatMenuItem, MatMenuTrigger, MatDivider],
    templateUrl: './search-toolbar.html',
    styleUrl: './search-toolbar.scss',
})
export class SearchToolbarComponent {
    protected readonly state = inject(SearchViewState);
    protected readonly SearchProvider = SearchProvider;
    protected readonly SearchDurationFilter = SearchDurationFilter;
    protected readonly SearchSortOption = SearchSortOption;
    protected readonly durationOptions = DURATION_OPTIONS;
    protected readonly sortOptions = SORT_OPTIONS;
}
