import { Component, inject, signal, computed } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { MatProgressSpinner } from '@angular/material/progress-spinner';
import { NotificationService } from '@app/services/notification.service';
import { finalize } from 'rxjs';
import { SearchViewService } from './search-view.service';
import { SearchTrackItemRdo } from './models/search-view.model';
import { PlayerService } from '@app/services/player.service';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';
import { MusicTracksViewService } from '@app/pages/music-tracks-view/music-tracks-view.service';
import { MusicCard } from '@shared/components/music-card/music-card';
import { SearchSkeletonComponent } from './components/search-skeleton/search-skeleton';
import { DurationPipe } from '@shared/pipes/duration.pipe';
import { MatIconButton } from '@angular/material/button';
import { EmptyPaneComponent } from '@shared/components/empty-pane/empty-pane';
import { ListViewShellComponent } from '@shared/components/list-view-shell/list-view-shell';
import { CompactNumberPipe } from '@shared/pipes/compact-number.pipe';
import { CdkMenuModule } from '@angular/cdk/menu';
import { FixedSizeVirtualScrollStrategy, RxVirtualFor, RxVirtualScrollViewportComponent, RxVirtualScrollWindowDirective } from '@rx-angular/template/virtual-scrolling';
import { SearchViewState } from './search-view.state';

@Component({
    selector: 'app-search-view',
    imports: [MusicCard, SearchSkeletonComponent, MatIcon, MatProgressSpinner, DurationPipe, MatIconButton, CompactNumberPipe, CdkMenuModule, RxVirtualScrollViewportComponent, RxVirtualFor, FixedSizeVirtualScrollStrategy, RxVirtualScrollWindowDirective, EmptyPaneComponent, ListViewShellComponent],
    templateUrl: './search-view.html',
    styleUrl: './search-view.scss',
    host: {
        '(window:scroll)': 'onWindowScroll()'
    }
})
export class SearchView {
    private readonly _api = inject(SearchViewService);
    private readonly _notification = inject(NotificationService);
    private readonly _tracksState = inject(MusicTracksViewState);
    private readonly _tracksService = inject(MusicTracksViewService);
    protected readonly player = inject(PlayerService);

    protected readonly state = inject(SearchViewState);

    protected readonly query = this.state.query;
    protected readonly results = this.state.results;
    protected readonly loading = this.state.loading;
    protected readonly hasMore = this.state.hasMore;
    protected readonly searched = this.state.searched;
    protected readonly isEmpty = this.state.isEmpty;

    protected readonly downloadingIds = signal<Set<number>>(new Set());
    protected readonly loadingStreamId = signal<number | null>(null);
    protected readonly downloadedIds = computed(() => new Set(this._tracksState.tracks().map(t => t.id)));

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

    protected onWindowScroll(): void {
        const threshold = 150;
        const scrollTop = window.scrollY || document.documentElement.scrollTop;
        const clientHeight = window.innerHeight;
        const scrollHeight = document.documentElement.scrollHeight;
        if (scrollHeight - scrollTop - clientHeight < threshold) {
            this.state.loadMore();
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
            error: () => this._notification.error('Failed to load audio preview'),
        });
    }

    protected downloadTrack(track: SearchTrackItemRdo): void {
        if (!track.permalinkUrl) {
            this._notification.error('Cannot download: missing track URL');
            return;
        }

        this._updateSet(this.downloadingIds, track.id, true);

        this._api.downloadTrack({ url: track.permalinkUrl }).pipe(
            finalize(() => this._updateSet(this.downloadingIds, track.id, false))
        ).subscribe({
            next: () => this._notification.success(`"${track.artist} – ${track.title}" queued`),
            error: (err) => this._notification.error(err?.error?.message || 'Download failed'),
        });
    }

    protected deleteTrack(track: SearchTrackItemRdo): void {
        this._tracksService.delete(track.id).subscribe({
            next: () => {
                this._tracksState.removeTrack(track.id);
                this._notification.success(`"${track.artist} – ${track.title}" deleted`);
            },
            error: () => this._notification.error('Failed to delete track'),
        });
    }
}
