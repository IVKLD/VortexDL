import { Component, inject, input } from '@angular/core';
import { NgOptimizedImage } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatDialog } from '@angular/material/dialog';
import { PlayableTrack } from '@shared/models/music-track.model';
import { MusicDetailModal } from '@app/pages/music-tracks-view/music-detail-modal/music-detail-modal';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';

@Component({
    selector: 'app-player-info',
    imports: [
        MatIcon,
        NgOptimizedImage
    ],
    templateUrl: './player-info.component.html',
    styleUrl: './player-info.component.scss',
})
export class PlayerInfoComponent {
    private readonly _dialog = inject(MatDialog);
    private readonly _tracksState = inject(MusicTracksViewState);

    public readonly data = input.required<PlayableTrack>();

    protected openDetailModal(): void {
        const track = this.data();
        const fullTrack = this._tracksState.tracks().find(t => t.id === track.id) ?? track;
        this._dialog.open(MusicDetailModal, { data: fullTrack, maxWidth: '500px', width: '100%' });
    }
}
