import {Component, input, output, inject} from '@angular/core';
import {MusicTrack} from '@shared/models/music-track.model';
import {MatIconButton} from '@angular/material/button';
import {NgOptimizedImage} from '@angular/common';
import {MatIcon} from '@angular/material/icon';
import {FileSizePipe} from '@shared/pipes/file-size.pipe';
import {outputFromObservable} from "@angular/core/rxjs-interop";
import {Subject} from "rxjs";
import {CdkMenuModule} from '@angular/cdk/menu';
import {OverlayContainer} from '@angular/cdk/overlay';

@Component({
    selector: 'app-music-card',
    imports: [MatIconButton, NgOptimizedImage, MatIcon, FileSizePipe, CdkMenuModule],
    templateUrl: './music-card.html',
    styleUrl: './music-card.scss',
    host: {
        '(click)': 'clickTrack.emit()',
        '[class.active]': 'isActive()',
        '[class.selected]': 'isSelected()',
        'role': 'button',
        'tabindex': '0'
    }
})
export class MusicCard {
    private readonly _overlayContainer = inject(OverlayContainer);

    public readonly track = input.required<MusicTrack>();
    public readonly isActive = input<boolean>(false);
    public readonly isPlaying = input<boolean>(false);
    public readonly isSelected = input<boolean>(false);

    public readonly playTrack = output<MusicTrack>();
    public readonly toggleSelect = output<void>();

    protected readonly _deleteTrackSubj = new Subject<void>();
    public readonly deleteTrack = outputFromObservable(this._deleteTrackSubj);

    public readonly clickTrack = output<void>();

    protected isMenuOpen(): boolean {
        return this._overlayContainer.getContainerElement().hasChildNodes();
    }

    protected onDeleteClick() {
        this._deleteTrackSubj.next();
    }
}


