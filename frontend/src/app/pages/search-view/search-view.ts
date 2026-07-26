import { Component, computed, inject } from '@angular/core';
import { MatProgressSpinner } from '@angular/material/progress-spinner';
import { NotificationService } from '@app/services/notification.service';
import { DownloadTrackingService } from '@app/services/download-tracking.service';
import { HeaderService } from '@shared/components/bricks/header/header.service';
import { SearchViewService } from './search-view.service';
import { SearchTrackItemRdo } from './models/search-view.model';
import { PlayerService } from '@app/services/player.service';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';
import { MusicTracksViewService } from '@app/pages/music-tracks-view/music-tracks-view.service';
import { SearchSkeletonComponent } from './components/search-skeleton/search-skeleton';
import { SearchHistoryPanelComponent } from './components/search-history-panel/search-history-panel';
import { SearchResultItemComponent } from './components/search-result-item/search-result-item';
import { EmptyPaneComponent } from '@shared/components/empty-pane/empty-pane';
import { ListViewShellComponent } from '@shared/components/list-view-shell/list-view-shell';
import {
    FixedSizeVirtualScrollStrategy,
    RxVirtualFor,
    RxVirtualScrollViewportComponent,
    RxVirtualScrollWindowDirective
} from '@rx-angular/template/virtual-scrolling';
import { SearchViewState } from './search-view.state';
import { SearchHistoryService } from './search-history.service';

@Component({
    selector: 'app-search-view',
    imports: [
        SearchResultItemComponent,
        SearchHistoryPanelComponent,
        SearchSkeletonComponent,
        MatProgressSpinner,
        RxVirtualScrollViewportComponent,
        RxVirtualFor,
        FixedSizeVirtualScrollStrategy,
        RxVirtualScrollWindowDirective,
        EmptyPaneComponent,
        ListViewShellComponent,
    ],
    templateUrl: './search-view.html',
    styleUrl: './search-view.scss',
})
export class SearchView {
    private readonly _api = inject(SearchViewService);
    private readonly _notification = inject(NotificationService);
    private readonly _tracksState = inject(MusicTracksViewState);
    private readonly _tracksService = inject(MusicTracksViewService);
    private readonly _headerService = inject(HeaderService);
    protected readonly tracking = inject(DownloadTrackingService);
    protected readonly player = inject(PlayerService);
    protected readonly history = inject(SearchHistoryService);

    protected readonly state = inject(SearchViewState);

    protected readonly isInitial = computed(() => {
        const bind = this._headerService.searchBind();
        return Boolean(bind?.focused?.()) || !this.state.query().trim();
    });
    protected readonly isEmpty = computed(() => !this.isInitial() && !this.state.results().length && !this.state.loading());

    protected readonly downloadedIds = computed(() => new Set(this._tracksState.tracks().map(t => t.id)));
    protected readonly downloadingIds = computed(() => new Set(this.tracking.activeDownloads().map(d => d.id)));

    protected onViewRange(range: { start: number; end: number }): void {
        if (this.isInitial() || !this.state.hasMore() || this.state.loading()) return;
        if (range.end >= this.state.results().length - 5) {
            this.state.loadMore();
        }
    }

    protected selectHistoryItem(item: string): void {
        this._headerService.searchBind()?.focused?.set(false);
        if (document.activeElement instanceof HTMLElement) {
            document.activeElement.blur();
        }
        this.state.search(item);
    }

    protected togglePlay(track: SearchTrackItemRdo): void {
        this.player.setQueue(this.state.results());
        this.player.play(track);
    }

    protected downloadTrack(track: SearchTrackItemRdo): void {
        if (!track.permalinkUrl) {
            this._notification.error('Cannot download: missing track URL');
            return;
        }

        this._api.downloadTrack({ url: track.permalinkUrl }).subscribe({
            next: () => this._notification.success(`"${track.artist} – ${track.title}" queued`),
            error: (err) => this._notification.error(err?.error?.message || 'Download failed'),
        });
    }

    protected deleteTrack(track: SearchTrackItemRdo): void {
        this._tracksService.delete(track.id).subscribe({
            next: () => this._tracksState.removeTrack(track.id),
            error: () => this._notification.error('Failed to delete track'),
        });
    }
}

