import {Component, input} from '@angular/core';
import {PlayerInfoComponent} from './components/player-info/player-info.component';
import {PlayerControlsComponent} from './components/player-controls/player-controls.component';
import {PlayerVolumeComponent} from './components/player-volume/player-volume.component';
import {PlayableTrack} from "@shared/models/music-track.model";

@Component({
    selector: 'app-player',
    imports: [
        PlayerInfoComponent,
        PlayerControlsComponent,
        PlayerVolumeComponent
    ],
    templateUrl: './player.html',
    styleUrl: './player.scss',
})
export class PlayerComponent {
    public readonly track = input.required<PlayableTrack>();
}
