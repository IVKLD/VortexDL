import { Component, computed, inject } from '@angular/core';
import { DatePipe, NgOptimizedImage } from '@angular/common';
import { MAT_DIALOG_DATA, MatDialogActions, MatDialogClose, MatDialogContent, MatDialogTitle } from '@angular/material/dialog';
import { AudioFormat, PlayableTrack } from '@shared/models/music-track.model';
import { MatIconButton } from '@angular/material/button';
import { MatIcon } from '@angular/material/icon';
import { MatTooltip } from '@angular/material/tooltip';
import { FileSizePipe } from '@shared/pipes/file-size.pipe';
import { PlayerService } from '@app/services/player.service';
import { TrackActionsService } from '@app/services/track-actions.service';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';

@Component({
    selector: 'app-music-detail-modal',
    imports: [
        MatIconButton,
        MatIcon,
        MatTooltip,
        NgOptimizedImage,
        MatDialogClose,
        MatDialogContent,
        MatDialogActions,
        FileSizePipe,
        DatePipe,
        MatDialogTitle,
    ],
    templateUrl: './music-detail-modal.html',
    styleUrl: './music-detail-modal.scss',
})
export class MusicDetailModal {
    protected readonly actions = inject(TrackActionsService);
    protected readonly state = inject(MusicTracksViewState);
    protected readonly player = inject(PlayerService);

    public readonly track: PlayableTrack = inject(MAT_DIALOG_DATA);
    protected readonly AudioFormat = AudioFormat;

    protected readonly isPlaying = computed(() => this.player.isTrackPlaying(this.track.id));
    protected readonly isLocalTrack = computed(() => this.state.isTrackLocal(this.track.id));
}
