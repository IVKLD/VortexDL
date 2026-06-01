import {ChangeDetectionStrategy, Component, input} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {SectionHeaderComponent} from '@shared/components/section-header/section-header';
import {Track} from '@shared/models/track.model';
import {MusicCard} from "@shared/components/music-card/music-card";

@Component({
    selector: 'app-recent-tracks',
    imports: [MatIcon, SectionHeaderComponent, MusicCard],
    templateUrl: './recent-tracks.component.html',
    styleUrl: './recent-tracks.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class RecentTracksComponent {
    public readonly tracks = input.required<Track[]>();
}
