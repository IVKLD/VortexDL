import { Injectable, inject, signal } from '@angular/core';
import { SearchViewService } from './search-view.service';
import { SearchTrackItemRdo } from './models/search-view.model';
import { finalize } from 'rxjs';
import { NotificationService } from '@app/services/notification.service';
import { SearchHistoryService } from './search-history.service';

@Injectable({ providedIn: 'root' })
export class SearchViewState {
    private readonly _api = inject(SearchViewService);
    private readonly _notification = inject(NotificationService);
    private readonly _history = inject(SearchHistoryService);

    public readonly query = signal<string>('');
    public readonly results = signal<SearchTrackItemRdo[]>([]);
    public readonly loading = signal(false);
    public readonly hasMore = signal(false);

    public search(query: string): void {
        const trimmed = query.trim();
        if (!trimmed) {
            this.query.set('');
            this.results.set([]);
            return;
        }

        this.query.set(trimmed);
        this._history.add(trimmed);

        this.loading.set(true);
        this.results.set([]);

        this._api.searchTracks(trimmed).pipe(
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
        if (!this.query().trim() || this.loading() || !this.hasMore()) return;

        this.loading.set(true);
        this._api.searchTracks(this.query().trim(), 20, this.results().length).pipe(
            finalize(() => this.loading.set(false))
        ).subscribe({
            next: (res) => {
                this.results.update(prev => [...prev, ...res.tracks]);
                this.hasMore.set(res.hasMore);
            },
        });
    }
}
