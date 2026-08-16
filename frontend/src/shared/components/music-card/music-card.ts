import { Component, input, output } from '@angular/core';
import { NgOptimizedImage } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatProgressSpinner } from '@angular/material/progress-spinner';

export interface MusicCardTrack {
    id: number;
    artist: string;
    title: string;
    artworkUrl?: string | null;
}

@Component({
    selector: 'app-music-card',
    imports: [NgOptimizedImage, MatIcon, MatProgressSpinner],
    templateUrl: './music-card.html',
    styleUrl: './music-card.scss',
    host: {
        '[class.active]': 'isActive() || isPlaying()',
        '[class.selected]': 'isSelected()',
        'role': 'button',
        'tabindex': '0'
    }
})
export class MusicCard {
    public readonly track = input.required<MusicCardTrack>();
    public readonly isActive = input<boolean>(false);
    public readonly isPlaying = input<boolean>(false);
    public readonly isSelected = input<boolean>(false);
    public readonly isLoadingStream = input<boolean>(false);

    public readonly playTrack = output<MusicCardTrack>();
}
