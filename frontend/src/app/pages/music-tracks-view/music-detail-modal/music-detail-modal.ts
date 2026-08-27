import {Component, inject} from '@angular/core';
import {DatePipe, NgOptimizedImage} from '@angular/common';
import {MAT_DIALOG_DATA, MatDialogClose} from '@angular/material/dialog';
import {AudioFormat, MusicTrack} from '@shared/models/music-track.model';
import {MatButton, MatIconButton} from '@angular/material/button';
import {MatIcon} from '@angular/material/icon';
import {FileSizePipe} from '@shared/pipes/file-size.pipe';

@Component({
    selector: 'app-music-detail-modal',
    imports: [MatButton, MatIconButton, MatIcon, NgOptimizedImage, MatDialogClose, FileSizePipe, DatePipe],
    templateUrl: './music-detail-modal.html',
    styleUrl: './music-detail-modal.scss',
})
export class MusicDetailModal {
    public readonly track: MusicTrack = inject(MAT_DIALOG_DATA);
    protected readonly AudioFormat = AudioFormat;
}


