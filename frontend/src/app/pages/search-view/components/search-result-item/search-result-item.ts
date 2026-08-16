import { Component, input, output } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { MatIconButton } from '@angular/material/button';
import { MatProgressSpinner } from '@angular/material/progress-spinner';
import { CdkMenuModule } from '@angular/cdk/menu';
import { MusicCard } from '@shared/components/music-card/music-card';
import { DurationPipe } from '@shared/pipes/duration.pipe';
import { CompactNumberPipe } from '@shared/pipes/compact-number.pipe';
import { PlatformChipComponent } from '@shared/components/platform-chip/platform-chip.component';
import { SearchTrackItemRdo } from '../../models/search-view.model';

@Component({
    selector: 'app-search-result-item',
    imports: [
        MusicCard,
        MatIcon,
        MatIconButton,
        MatProgressSpinner,
        DurationPipe,
        CompactNumberPipe,
        CdkMenuModule,
        PlatformChipComponent,
    ],
    templateUrl: './search-result-item.html',
    styleUrl: './search-result-item.scss',
})
export class SearchResultItemComponent {
    readonly track = input.required<SearchTrackItemRdo>();
    readonly isPlaying = input<boolean>(false);
    readonly isLoadingStream = input<boolean>(false);
    readonly isDownloaded = input<boolean>(false);
    readonly isDownloading = input<boolean>(false);

    readonly playTrack = output<SearchTrackItemRdo>();
    readonly downloadTrack = output<SearchTrackItemRdo>();
    readonly deleteTrack = output<SearchTrackItemRdo>();
}
