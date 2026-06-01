import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { PlayerService } from '@app/services/player.service';
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
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PlayerComponent {
    protected readonly player = inject(PlayerService);
}
