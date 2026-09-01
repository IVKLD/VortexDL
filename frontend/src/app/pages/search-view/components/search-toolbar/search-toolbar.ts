import {Component, inject} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {CdkMenuModule} from '@angular/cdk/menu';
import {SearchViewState} from '../../search-view.state';
import {SearchDurationFilter, SearchProvider} from '../../models/search-view.model';
import {MatButton} from "@angular/material/button";

@Component({
    selector: 'app-search-toolbar',
    imports: [MatIcon, CdkMenuModule, MatButton],
    templateUrl: './search-toolbar.html',
    styleUrl: './search-toolbar.scss',
})
export class SearchToolbarComponent {
    protected readonly state = inject(SearchViewState);
    protected readonly SearchProvider = SearchProvider;
    protected readonly SearchDurationFilter = SearchDurationFilter;

    protected setProvider(provider: SearchProvider): void {
        this.state.setProvider(provider);
    }

    protected setDuration(duration: SearchDurationFilter): void {
        this.state.setDuration(duration);
    }
}
