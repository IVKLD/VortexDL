import { Component, computed, inject } from '@angular/core';
import { MusicTrack } from '@shared/models/music-track.model';
import { MusicCard } from '@shared/components/music-card/music-card';
import { MusicTracksViewState } from './music-tracks-view.state';
import { MusicTracksViewService } from './music-tracks-view.service';
import { PlayerService } from '@app/services/player.service';
import {
    FixedSizeVirtualScrollStrategy,
    RxVirtualFor,
    RxVirtualScrollViewportComponent,
    RxVirtualScrollWindowDirective
} from '@rx-angular/template/virtual-scrolling';
import { MatDialog } from '@angular/material/dialog';
import { MusicDetailModal } from './music-detail-modal/music-detail-modal';
import { MatIcon } from '@angular/material/icon';
import { SelectionBar } from '@shared/components/selection-bar/selection-bar';
import { CdkMenuModule } from '@angular/cdk/menu';
import { OverlayContainer } from '@angular/cdk/overlay';
import { EmptyPaneComponent } from '@shared/components/empty-pane/empty-pane';
import { SearchSkeletonComponent } from '../search-view/components/search-skeleton/search-skeleton';
import { ListViewShellComponent } from '@shared/components/list-view-shell/list-view-shell';
import { FileSizePipe } from '@shared/pipes/file-size.pipe';
import { MatIconButton } from '@angular/material/button';
import { DragSelectDirective } from './directives/drag-select.directive';
import { PlatformChipComponent } from '@shared/components/platform-chip/platform-chip.component';
import { PlatformPipe } from '@shared/pipes/platform.pipe';

import { AudioFormat } from '@shared/models/music-track.model';

@Component({
    selector: 'app-music-tracks-view',
    imports: [
        MusicCard,
        RxVirtualScrollViewportComponent,
        RxVirtualFor,
        FixedSizeVirtualScrollStrategy,
        RxVirtualScrollWindowDirective,
        MatIcon,
        SelectionBar,
        CdkMenuModule,
        FileSizePipe,
        MatIconButton,
        SearchSkeletonComponent,
        EmptyPaneComponent,
        ListViewShellComponent,
        DragSelectDirective,
        PlatformChipComponent,
        PlatformPipe,
    ],
    templateUrl: './music-tracks-view.html',
    styleUrl: './music-tracks-view.scss',
})
export class MusicTracksView {
    private readonly _api = inject(MusicTracksViewService);
    private readonly _dialog = inject(MatDialog);
    private readonly _overlayContainer = inject(OverlayContainer);
    protected readonly state = inject(MusicTracksViewState);
    protected readonly player = inject(PlayerService);

    protected readonly AudioFormat = AudioFormat;
    protected readonly tracks = this.state.sortedTracks;

    protected readonly activeTrackId = computed(() => this.player.currentTrack()?.id);

    constructor() {
        this._api.indexing().subscribe();
    }

    protected playTrack(track: MusicTrack): void {
        this.player.setQueue(this.tracks());
        this.player.play(track);
    }

    protected handleCardClick(track: MusicTrack): void {
        if (this._overlayContainer.getContainerElement().hasChildNodes()) {
            return;
        }
        this._dialog.open(MusicDetailModal, { data: track, maxWidth: '500px', width: '100%' });
    }

    protected deleteTrack(track: MusicTrack): void {
        this._api.delete(track.id).subscribe({
            next: () => {
                this.state.removeTrack(track.id);
                this.player.removeFromQueue(track.id);
            },
        });
    }

    protected downloadTrack(track: MusicTrack): void {
        const link = document.createElement('a');
        link.href = `/api/stream/${track.id}`;
        link.download = `${track.artist} - ${track.title}.${track.format}`;
        link.click();
    }

    protected deleteSelected(): void {
        this.state.selectedTracks().forEach(track => this.deleteTrack(track));
    }
}
