import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { UpperCasePipe, NgOptimizedImage } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { RouterLink } from '@angular/router';

export interface RecentTrack {
    id: number;
    filename: string;
    format: string;
    artworkUrl?: string;
}

@Component({
    selector: 'app-recent-tracks',
    imports: [UpperCasePipe, MatIcon, RouterLink, NgOptimizedImage],
    templateUrl: './recent-tracks.component.html',
    styleUrl: './recent-tracks.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class RecentTracksComponent {
    tracks = input.required<RecentTrack[]>();
}
