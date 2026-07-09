import { Injectable, inject, signal, computed } from '@angular/core';
import { SearchViewService } from './search-view.service';
import { SearchTrackItemRdo } from './models/search-view.model';
import { MusicTracksViewState } from '../music-tracks-view/music-tracks-view.state';
import { finalize } from 'rxjs';
import { NotificationService } from '@app/services/notification.service';

@Injectable({ providedIn: 'root' })
export class SearchViewState {
    private readonly _api = inject(SearchViewService);
    private readonly _notification = inject(NotificationService);
    private readonly _tracksState = inject(MusicTracksViewState);

    public readonly query = signal('');
    public readonly results = signal<SearchTrackItemRdo[]>([]);
    public readonly loading = signal(false);
    public readonly hasMore = signal(false);
    public readonly searched = signal(false);
    public readonly activeQuery = signal('');

    public readonly isEmpty = computed(() => this.searched() && !this.results().length && !this.loading());

    public search(query: string): void {
        this.query.set(query);
        this.activeQuery.set(query);
        if (this.loading()) return;

        this.loading.set(true);
        this.searched.set(true);
        this.results.set([]);

        this._api.searchTracks(query).pipe(
            finalize(() => this.loading.set(false))
        ).subscribe({
            next: (res) => {
                this.results.set(res.tracks);
                this.hasMore.set(res.hasMore);
            },
            error: () => {
                this._notification.error('Search failed. Try again.');
            },
        });
    }

    public loadMore(): void {
        if (this.loading() || !this.hasMore()) return;

        this.loading.set(true);
        this._api.searchTracks(this.activeQuery(), 20, this.results().length).pipe(
            finalize(() => this.loading.set(false))
        ).subscribe({
            next: (res) => {
                this.results.update(prev => [...prev, ...res.tracks]);
                this.hasMore.set(res.hasMore);
            },
        });
    }

    public clearSearch(): void {
        this.query.set('');
        this.activeQuery.set('');
        this.results.set([]);
        this.searched.set(false);
    }
}
