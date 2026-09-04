import { Component, inject } from '@angular/core';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';
import { PlayerService } from '@app/services/player.service';
import { TrackActionsService } from '@app/services/track-actions.service';
import { PlayerControlsComponent } from './components/player-controls/player-controls.component';
import { PlayerInfoComponent } from './components/player-info/player-info.component';
import { PlayerVolumeComponent } from './components/player-volume/player-volume.component';
import { TrackActions } from './components/track-actions/track-actions';

@Component({
    selector: 'app-player',
    imports: [
        PlayerInfoComponent,
        PlayerControlsComponent,
        TrackActions,
        PlayerVolumeComponent
    ],
    templateUrl: './player.html',
    styleUrl: './player.scss',
})
export class PlayerComponent {
    protected readonly actions = inject(TrackActionsService);
    protected readonly player = inject(PlayerService);
    protected readonly tracksState = inject(MusicTracksViewState);
}
