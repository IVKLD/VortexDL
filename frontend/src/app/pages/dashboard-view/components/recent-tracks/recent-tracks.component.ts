import {ChangeDetectionStrategy, Component, input, output} from '@angular/core';
import {NgOptimizedImage} from '@angular/common';
import {MatIcon} from '@angular/material/icon';
import {SectionHeaderComponent} from '@shared/components/section-header/section-header';
import {MatIconButton} from '@angular/material/button';
import {Track} from '@shared/models/track.model';

@Component({
    selector: 'app-recent-tracks',
    imports: [MatIcon, NgOptimizedImage, SectionHeaderComponent, MatIconButton],
    templateUrl: './recent-tracks.component.html',
    styleUrl: './recent-tracks.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class RecentTracksComponent {
    public readonly tracks = input.required<Track[]>();
    public readonly playTrack = output<Track>();
}
