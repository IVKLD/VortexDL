import {ChangeDetectionStrategy, Component, effect, inject, untracked} from '@angular/core';
import {MusicTracksViewService} from './music-tracks-view.service';
import {Track} from '@shared/models/track.model';
import {MusicCard} from '@shared/components/music-card/music-card';
import {MusicTracksViewState} from './music-tracks-view.state';
import {PlayerService} from '@app/services/player.service';
import {
    FixedSizeVirtualScrollStrategy,
    RxVirtualFor,
    RxVirtualScrollViewportComponent,
} from '@rx-angular/template/virtual-scrolling';
import {MatDialog} from '@angular/material/dialog';
import {MusicDetailModal} from './music-detail-modal/music-detail-modal';
import {MatIcon} from '@angular/material/icon';

@Component({
    selector: 'app-music-tracks-view',
    imports: [MusicCard, RxVirtualScrollViewportComponent, RxVirtualFor, FixedSizeVirtualScrollStrategy, MatIcon],
    providers: [MusicTracksViewService],
    templateUrl: './music-tracks-view.html',
    styleUrl: './music-tracks-view.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MusicTracksView {
    private readonly _api = inject(MusicTracksViewService);
    private readonly _state = inject(MusicTracksViewState);
    private readonly _dialog = inject(MatDialog);

    protected readonly player = inject(PlayerService);
    protected readonly tracks = this._state.sortedTracks;

    constructor() {
        this._api.index().subscribe();

        effect(() => {
            const option = this._state.sortOption();
            untracked(() => {
                this._state.startLoading();
                const [sort, order] = option.split('-');

                this._api.getAll(sort, order)
                    .subscribe({
                    next: tracks => this._state.setTracks = tracks
                });
            });
        });
    }

    protected playTrack(track: Track) {
        this.player.queue.set(this.tracks());
        this.player.play(track);
    }

    protected deleteMusic(track: Track) {
        this._api.delete(track.id).subscribe({
            next: () => {
                this._state.removeTrack(track);
            },
        });
    }

    protected openDetail(track: Track) {
        this._dialog.open(MusicDetailModal, {
            data: track,
            maxWidth: '500px',
            width: '100%',
        });
    }
}
