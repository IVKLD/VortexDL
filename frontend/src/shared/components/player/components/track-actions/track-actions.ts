import { Component, input, output } from '@angular/core';
import { MatIconButton } from '@angular/material/button';
import { MatIcon } from '@angular/material/icon';
import { MatTooltip } from '@angular/material/tooltip';
import { PlayableTrack } from '@shared/models/music-track.model';

@Component({
    selector: 'app-track-actions',
    imports: [MatIcon, MatIconButton, MatTooltip],
    templateUrl: './track-actions.html',
    styleUrl: './track-actions.scss',
})
export class TrackActions {
    public readonly track = input.required<PlayableTrack>();
    public readonly shuffle = input<boolean>(false);
    public readonly isLocal = input<boolean>(false);

    public readonly shuffleChange = output<void>();
    public readonly downloadTrack = output<PlayableTrack>();
    public readonly openSource = output<string>();
    public readonly deleteTrack = output<PlayableTrack>();
}
