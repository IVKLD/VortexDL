import { Component, inject, signal, computed } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { MatProgressSpinner } from '@angular/material/progress-spinner';
import { MatSnackBar } from '@angular/material/snack-bar';
import { finalize } from 'rxjs';
import { SearchViewService } from './search-view.service';
import { SearchTrackItemRdo } from './models/search-view.model';
import { PlayerService } from '@app/services/player.service';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';
import { MusicTracksViewService } from '@app/pages/music-tracks-view/music-tracks-view.service';
import { MusicCard } from '@shared/components/music-card/music-card';
import { HeaderService } from '@shared/components/bricks/header/header.service';
import { SearchSkeletonComponent } from './components/search-skeleton/search-skeleton';
import { DurationPipe } from '@shared/pipes/duration.pipe';
import { MatIconButton } from '@angular/material/button';
import { CompactNumberPipe } from '@shared/pipes/compact-number.pipe';
import { CdkMenuModule } from '@angular/cdk/menu';
import { FixedSizeVirtualScrollStrategy, RxVirtualFor, RxVirtualScrollViewportComponent, RxVirtualScrollWindowDirective } from '@rx-angular/template/virtual-scrolling';
import { form } from '@angular/forms/signals';

@Component({
    selector: 'app-search-view',
    imports: [MusicCard, SearchSkeletonComponent, MatIcon, MatProgressSpinner, DurationPipe, MatIconButton, CompactNumberPipe, CdkMenuModule, RxVirtualScrollViewportComponent, RxVirtualFor, FixedSizeVirtualScrollStrategy, RxVirtualScrollWindowDirective],
    templateUrl: './search-view.html',
    styleUrl: './search-view.scss',
    host: {
        '(window:scroll)': 'onWindowScroll()'
    }
})
export class SearchView {
    private readonly _api = inject(SearchViewService);
    private readonly _snackBar = inject(MatSnackBar);
    private readonly _tracksState = inject(MusicTracksViewState);
    private readonly _tracksService = inject(MusicTracksViewService);
    private readonly _headerService = inject(HeaderService);
    protected readonly player = inject(PlayerService);
    private readonly _activeQuery = signal('');

    protected readonly query = signal('');
    protected readonly searchForm = form(this.query);
    protected readonly results = signal<SearchTrackItemRdo[]>([]);
    protected readonly loading = signal(false);
    protected readonly hasMore = signal(false);
    protected readonly searched = signal(false);
    protected readonly downloadingIds = signal<Set<number>>(new Set());
    protected readonly loadingStreamId = signal<number | null>(null);
    protected readonly downloadedIds = computed(() => new Set(this._tracksState.tracks().map(t => t.id)));

    protected readonly isEmpty = computed(() => this.searched() && !this.results().length && !this.loading());

    constructor() {
        this._headerService.bindSearch({
            formField: this.searchForm,
            onSubmit: (q) => this.search(q),
            onClear: () => this.clearSearch(),
        });
    }

    private _updateSet(sig: ReturnType<typeof signal<Set<number>>>, id: number, add: boolean): void {
        sig.update(set => {
            const next = new Set(set);
            if (add) {
                next.add(id);
            } else {
                next.delete(id);
            }
            return next;
        });
    }

    protected search(query: string): void {
        this.query.set(query);
        this._activeQuery.set(query);
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
                this._snackBar.open('Search failed. Try again.', 'OK');
            },
        });
    }

    protected loadMore(): void {
        if (this.loading() || !this.hasMore()) return;

        this.loading.set(true);
        this._api.searchTracks(this._activeQuery(), 20, this.results().length).pipe(
            finalize(() => this.loading.set(false))
        ).subscribe({
            next: (res) => {
                this.results.update(prev => [...prev, ...res.tracks]);
                this.hasMore.set(res.hasMore);
            },
        });
    }

    protected onWindowScroll(): void {
        const threshold = 150;
        const scrollTop = window.scrollY || document.documentElement.scrollTop;
        const clientHeight = window.innerHeight;
        const scrollHeight = document.documentElement.scrollHeight;
        if (scrollHeight - scrollTop - clientHeight < threshold) {
            this.loadMore();
        }
    }

    protected togglePlay(track: SearchTrackItemRdo): void {
        if (this.player.currentTrack()?.id === track.id) {
            this.player.togglePlay();
            return;
        }

        this.loadingStreamId.set(track.id);

        this._api.getStreamUrl(track.id).pipe(
            finalize(() => this.loadingStreamId.set(null))
        ).subscribe({
            next: (res) => {
                this.player.play({
                    id: track.id,
                    artist: track.artist,
                    title: track.title,
                    artworkUrl: track.artworkUrl,
                    sourceUrl: track.permalinkUrl,
                }, res.url);
            },
            error: () => this._snackBar.open('Failed to load audio preview', 'OK'),
        });
    }

    protected downloadTrack(track: SearchTrackItemRdo): void {
        if (!track.permalinkUrl) {
            this._snackBar.open('Cannot download: missing track URL', 'OK');
            return;
        }

        this._updateSet(this.downloadingIds, track.id, true);

        this._api.downloadTrack({ url: track.permalinkUrl }).pipe(
            finalize(() => this._updateSet(this.downloadingIds, track.id, false))
        ).subscribe({
            next: () => this._snackBar.open(`"${track.artist} – ${track.title}" queued`, 'OK'),
            error: (err) => this._snackBar.open(err?.error?.message || 'Download failed', 'OK'),
        });
    }

    protected deleteTrack(track: SearchTrackItemRdo): void {
        this._tracksService.delete(track.id).subscribe({
            next: () => {
                this._tracksState.removeTrack(track.id);
                this._snackBar.open(`"${track.artist} – ${track.title}" deleted`, 'OK');
            },
            error: () => this._snackBar.open('Failed to delete track', 'OK'),
        });
    }

    protected clearSearch(): void {
        this.query.set('');
        this._activeQuery.set('');
        this.results.set([]);
        this.searched.set(false);
    }
}
