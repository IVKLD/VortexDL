import { ChangeDetectionStrategy, Component, inject, input, signal } from '@angular/core';
import { MatFormField, MatPrefix } from '@angular/material/form-field';
import { MatInput } from '@angular/material/input';
import { MatIcon } from '@angular/material/icon';
import { distinctUntilChanged } from 'rxjs';
import { takeUntilDestroyed, toObservable } from '@angular/core/rxjs-interop';
import { debounce, form, FormField } from '@angular/forms/signals';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';
import { HeaderConfig, HeaderFeature } from '../../header.types';

@Component({
    selector: 'app-header-center',
    imports: [MatFormField, MatInput, MatPrefix, MatIcon, FormField],
    templateUrl: './header-center.component.html',
    styleUrl: './header-center.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class HeaderCenter {
    public readonly config = input<HeaderConfig>();
    private readonly _musicState = inject(MusicTracksViewState);
    protected readonly Feature = HeaderFeature;

    private readonly _searchValue = signal(this._musicState.searchQuery());
    protected readonly searchForm = form(this._searchValue, (p) => {
        debounce(p, 200);
    });

    constructor() {
        toObservable(this.searchForm().value).pipe(
            distinctUntilChanged(),
            takeUntilDestroyed()
        ).subscribe(query => this._musicState.setSearchQuery(query));
    }

    protected hasFeature(feature: HeaderFeature): boolean {
        return this.config()?.features.includes(feature) ?? false;
    }
}
