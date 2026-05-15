import {ChangeDetectionStrategy, Component, computed, input, output} from '@angular/core';
import {Track} from '@shared/models/track.model';
import {MatIconButton} from '@angular/material/button';
import {NgOptimizedImage} from '@angular/common';
import {MatIcon} from '@angular/material/icon';
import {FileSizePipe} from '@shared/pipes/file-size.pipe';

@Component({
    selector: 'app-music-card',
    imports: [MatIconButton, NgOptimizedImage, MatIcon, FileSizePipe],
    templateUrl: './music-card.html',
    styleUrl: './music-card.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MusicCard {
    public readonly track = input.required<Track>();
    public readonly isActive = input<boolean>(false);
    public readonly isPlaying = input<boolean>(false);
    public readonly playTrack = output<Track>();
    public readonly deleteMusic = output<void>();
    public readonly trackClick = output<void>();

    protected readonly displayTitle = computed(() => {
        const item = this.track();
        return `${item.artist} - ${item.title}`;
    });
}
