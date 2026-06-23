import {Component, input} from '@angular/core';
import {MusicTrack} from '@shared/models/music-track.model';
import { PlayerInfoComponent } from './components/player-info/player-info.component';
import { PlayerControlsComponent } from './components/player-controls/player-controls.component';
import { PlayerVolumeComponent } from './components/player-volume/player-volume.component';

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
    public readonly track = input.required<MusicTrack>();
}
