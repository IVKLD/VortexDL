import {ChangeDetectionStrategy, Component, input, output} from '@angular/core';
import {Track} from '@shared/models/track.model';
import {MatIconButton} from '@angular/material/button';
import {NgOptimizedImage} from '@angular/common';
import {MatIcon} from '@angular/material/icon';
import {FileSizePipe} from '@shared/pipes/file-size.pipe';
import {outputFromObservable} from "@angular/core/rxjs-interop";
import {Subject} from "rxjs";

@Component({
    selector: 'app-music-card',
    imports: [MatIconButton, NgOptimizedImage, MatIcon, FileSizePipe],
    templateUrl: './music-card.html',
    styleUrl: './music-card.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
    host: {
        '(click)': 'clickTrack.emit()',
        '[class.active]': 'isActive()',
        'role': 'button',
        'tabindex': '0'
    }
})
export class MusicCard {
    public readonly track = input.required<Track>();
    public readonly isActive = input<boolean>(false);
    public readonly isPlaying = input<boolean>(false);
    public readonly playTrack = output<Track>();

    protected readonly _deleteTrackSubj = new Subject<void>();
    public readonly deleteTrack = outputFromObservable(this._deleteTrackSubj);

    public readonly clickTrack = output<void>();

    protected onDeleteClick() {
        this._deleteTrackSubj.next();
    }
}
