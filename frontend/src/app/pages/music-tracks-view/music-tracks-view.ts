import { Component, inject, computed } from '@angular/core';
import { MusicTrack } from '@shared/models/music-track.model';
import { MusicCard } from '@shared/components/music-card/music-card';
import { MusicTracksViewState } from './music-tracks-view.state';
import { MusicTracksViewService } from './music-tracks-view.service';
import { PlayerService } from '@app/services/player.service';
import { FixedSizeVirtualScrollStrategy, RxVirtualFor, RxVirtualScrollViewportComponent } from '@rx-angular/template/virtual-scrolling';
import { MatDialog } from '@angular/material/dialog';
import { MusicDetailModal } from './music-detail-modal/music-detail-modal';
import { MatIcon } from '@angular/material/icon';
import { SelectionBar } from '@shared/components/selection-bar/selection-bar';
import { CdkMenuModule } from '@angular/cdk/menu';
import { OverlayContainer } from '@angular/cdk/overlay';

@Component({
    selector: 'app-music-tracks-view',
    imports: [MusicCard, RxVirtualScrollViewportComponent, RxVirtualFor, FixedSizeVirtualScrollStrategy, MatIcon, SelectionBar, CdkMenuModule],
    templateUrl: './music-tracks-view.html',
    styleUrl: './music-tracks-view.scss',
})
export class MusicTracksView {
    private readonly _api = inject(MusicTracksViewService);
    private readonly _dialog = inject(MatDialog);
    private readonly _overlayContainer = inject(OverlayContainer);

    protected readonly state = inject(MusicTracksViewState);
    protected readonly player = inject(PlayerService);
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

        if (this.state.hasSelection()) {
            this.state.toggleSelect(track);
        } else {
            this._dialog.open(MusicDetailModal, { data: track, maxWidth: '500px', width: '100%' });
        }
    }

    protected deleteTrack(track: MusicTrack): void {
        this._api.delete(track.id).subscribe({
            next: () => {
                this.state.removeTrack(track);
                this.player.removeFromQueue(track.id);
            },
        });
    }

    protected deleteSelected(): void {
        this.state.selectedTracks().forEach(track => this.deleteTrack(track));
    }
}
