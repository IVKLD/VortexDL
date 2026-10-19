import {Component, computed, inject} from '@angular/core';
import {AudioFormat, MusicTrack} from '@shared/models/music-track.model';
import {MusicCard} from '@shared/components/music-card/music-card';
import {MusicTracksViewState} from './music-tracks-view.state';
import {PlayerService} from '@app/services/player.service';
import {TrackActionsService} from '@app/services/track-actions.service';
import {
    FixedSizeVirtualScrollStrategy,
    RxVirtualFor,
    RxVirtualScrollViewportComponent,
    RxVirtualScrollWindowDirective
} from '@rx-angular/template/virtual-scrolling';
import {MatIcon} from '@angular/material/icon';
import {SelectionBar} from '@shared/components/selection-bar/selection-bar';
import {MatMenuModule} from '@angular/material/menu';
import {EmptyPaneComponent} from '@shared/components/empty-pane/empty-pane';
import {SearchSkeletonComponent} from '../search-view/components/search-skeleton/search-skeleton';
import {ListViewShellComponent} from '@shared/components/list-view-shell/list-view-shell';
import {FileSizePipe} from '@shared/pipes/file-size.pipe';
import {MatIconButton} from '@angular/material/button';
import {DragSelectDirective} from '@shared/directives/drag-select/drag-select.directive';
import {PlatformChipComponent} from '@shared/components/platform-chip/platform-chip.component';
import {PlatformPipe} from '@shared/pipes/platform.pipe';

@Component({
    selector: 'app-music-tracks-view',
    imports: [
        MusicCard,
        RxVirtualScrollViewportComponent,
        RxVirtualFor,
        FixedSizeVirtualScrollStrategy,
        RxVirtualScrollWindowDirective,
        MatIcon,
        SelectionBar,
        MatMenuModule,
        FileSizePipe,
        MatIconButton,
        SearchSkeletonComponent,
        EmptyPaneComponent,
        ListViewShellComponent,
        DragSelectDirective,
        PlatformChipComponent,
        PlatformPipe,
    ],
    templateUrl: './music-tracks-view.html',
    styleUrl: './music-tracks-view.scss',
})
export class MusicTracksView {
    protected readonly actions = inject(TrackActionsService);
    protected readonly state = inject(MusicTracksViewState);
    protected readonly player = inject(PlayerService);

    protected readonly AudioFormat = AudioFormat;
    protected readonly tracks = this.state.sortedTracks;

    protected readonly activeTrackId = computed(() => this.player.currentTrack()?.id);

    protected playTrack(track: MusicTrack): void {
        this.player.setQueue(this.tracks());
        this.player.play(track);
    }

    protected handleCardClick(track: MusicTrack): void {
        this.actions.openDetails(track);
    }

    protected deleteSelected(): void {
        this.actions.delete(this.state.selection.ids());
    }
}
