import { Injectable, inject, signal } from '@angular/core';
import { SearchViewService } from './search-view.service';
import { SearchTrackItemRdo } from './models/search-view.model';
import { finalize } from 'rxjs';
import { NotificationService } from '@app/services/notification.service';
import { SearchHistoryService } from './search-history.service';
import { SettingsState } from '@app/pages/settings-view/settings.state';
import { HeaderService } from '@shared/components/bricks/header/header.service';

@Injectable({ providedIn: 'root' })
export class SearchViewState {
    private readonly _api = inject(SearchViewService);
    private readonly _notification = inject(NotificationService);
    private readonly _history = inject(SearchHistoryService);
    private readonly _settingsState = inject(SettingsState);
    private readonly _headerService = inject(HeaderService);

    public readonly query = signal<string>('');

    public readonly hasSearched = signal<boolean>(false);
    public readonly showHistory = signal<boolean>(false);
    public readonly results = signal<SearchTrackItemRdo[]>([]);
    public readonly loading = signal(false);
    public readonly hasMore = signal(false);

    public clear(): void {
        this.query.set('');
        this.hasSearched.set(false);
        this.showHistory.set(false);
        this.results.set([]);
        this.loading.set(false);
        this.hasMore.set(false);
    }

    public search(query: string): void {
        const trimmed = query.trim();
        if (!trimmed) {
            this.clear();
            return;
        }

        this.query.set(trimmed);
        this.hasSearched.set(true);
        this.showHistory.set(false);
        this._history.add(trimmed);
        this.results.set([]);

        this.loading.set(true);
        const limit = this._settingsState.settingsModel().system?.limitPerPage ?? 20;
        this._api.searchTracks(trimmed, limit).pipe(
            finalize(() => this.loading.set(false))
        ).subscribe({
            next: (res) => {
                this.results.set(res.tracks);
                this.hasMore.set(res.hasMore);
            },
            error: () => this._notification.error('Search failed. Try again.'),
        });
    }

    public loadMore(): void {
        const q = this.query().trim();
        if (!q || this.loading() || !this.hasMore() || this.results().length === 0) return;

        this.loading.set(true);
        const limit = this._settingsState.settingsModel().system?.limitPerPage ?? 20;
        this._api.searchTracks(q, limit, this.results().length).pipe(
            finalize(() => this.loading.set(false))
        ).subscribe({
            next: (res) => {
                this.results.update(prev => [...prev, ...res.tracks]);
                this.hasMore.set(res.hasMore);
            },
        });
    }
}
