import {ChangeDetectionStrategy, Component, computed, input, output} from '@angular/core';
import {Track} from '../music-tracks-view.service';
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
    public track = input.required<Track>();
    public deleteMusic = output();
    public trackClick = output();

    protected readonly displayTitle = computed(() => {
        const item = this.track();
        return item.filename.replace(`.${item.format}`, '').replace(/_/g, ' ');
    });
}
