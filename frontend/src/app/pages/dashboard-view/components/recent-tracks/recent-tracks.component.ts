import {ChangeDetectionStrategy, Component, input} from '@angular/core';
import {NgOptimizedImage, UpperCasePipe} from '@angular/common';
import {MatIcon} from '@angular/material/icon';
import {RouterLink} from '@angular/router';
import {RecentTrack} from '../../dashboard-view.model';

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
