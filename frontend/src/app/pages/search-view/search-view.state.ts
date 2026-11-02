import { computed, inject, Injectable, signal } from '@angular/core';
import { Observable } from 'rxjs';
import { NotificationService } from '@app/services/notification.service';
import { LoadingService } from '@app/services/loading.service';
import { SettingsState } from '@app/pages/settings-view/settings.state';
import {
    DEFAULT_SEARCH_SETTINGS,
    DURATION_OPTIONS,
    SORT_OPTIONS,
    SearchDurationFilter,
    SearchProvider,
    SearchRdo,
    SearchSettingsModel,
    SearchSortOption,
    SearchTrackItemRdo,
} from './models';
import { SearchHistoryService } from './search-history.service';
import { SearchFilterService } from './search-filter.service';
import { SearchViewService } from './search-view.service';

@Injectable({ providedIn: 'root' })
export class SearchViewState {
    private readonly _api = inject(SearchViewService);
    private readonly _filter = inject(SearchFilterService);
    private readonly _history = inject(SearchHistoryService);
    private readonly _notification = inject(NotificationService);
    private readonly _settingsState = inject(SettingsState);

    public readonly query = signal<string>('');
    public readonly hasSearched = signal<boolean>(false);
    public readonly showHistory = signal<boolean>(false);
    public readonly rawTracks = signal<SearchTrackItemRdo[]>([]);
    public readonly loading = inject(LoadingService).isLoading;
    public readonly hasMore = signal<boolean>(false);

    public readonly config = signal<SearchSettingsModel>(DEFAULT_SEARCH_SETTINGS);

    public readonly durationLabel = computed(() => {
        const dur = this.config().duration;
        return DURATION_OPTIONS.find((o) => o.value === dur)?.shortLabel ?? 'Duration';
    });

    public readonly sortLabel = computed(() => {
        const sort = this.config().sort;
        return SORT_OPTIONS.find((o) => o.value === sort)?.shortLabel ?? 'Sort';
    });

    public readonly results = computed(() => {
        const { duration, sort } = this.config();
        return this._filter.apply(this.rawTracks(), duration, sort);
    });

    private requestTracks(query: string, offset: number): Observable<SearchRdo> {
        const limit = this._settingsState.settingsModel().system?.limitPerPage ?? 20;
        const { provider, duration } = this.config();
        const dur = provider === SearchProvider.YouTube ? duration : undefined;

        return this._api.searchTracks(query, limit, offset, provider, dur);
    }

    public setProvider(provider: SearchProvider) {
        if (this.config().provider === provider) return;
        this.config.update((c) => ({ ...c, provider }));
        if (this.hasSearched() && this.query()) {
            this.search(this.query());
        }
    }

    public setDuration(duration: SearchDurationFilter) {
        if (this.config().duration === duration) return;
        this.config.update((c) => ({ ...c, duration }));
        if (this.config().provider === SearchProvider.YouTube && this.hasSearched() && this.query()) {
            this.search(this.query());
        }
    }

    public setSortOption(sort: SearchSortOption) {
        this.config.update((c) => ({ ...c, sort }));
    }

    public clear() {
        this.query.set('');
        this.hasSearched.set(false);
        this.showHistory.set(false);
        this.rawTracks.set([]);
        this.hasMore.set(false);
    }

    public search(query: string) {
        const trimmed = query.trim();
        if (!trimmed) {
            this.clear();
            return;
        }

        this.query.set(trimmed);
        this.hasSearched.set(true);
        this.showHistory.set(false);
        this._history.add(trimmed);
        this.rawTracks.set([]);

        this.requestTracks(trimmed, 0).subscribe({
            next: (res) => {
                this.rawTracks.set(res.tracks);
                this.hasMore.set(res.hasMore);
            },
            error: () => this._notification.error('Search failed. Try again.'),
        });
    }

    public loadMore() {
        const q = this.query().trim();
        if (!q || this.loading() || !this.hasMore() || this.rawTracks().length === 0) return;

        this.requestTracks(q, this.rawTracks().length).subscribe({
            next: (res) => {
                const existingIds = new Set(this.rawTracks().map((t) => t.id));
                const unique = res.tracks.filter((t) => !existingIds.has(t.id));
                this.rawTracks.update((prev) => [...prev, ...unique]);
                this.hasMore.set(res.hasMore);
            },
        });
    }
}
