import {Component, inject} from '@angular/core';
import { MatIconButton } from '@angular/material/button';
import { MatIcon } from '@angular/material/icon';
import { PlayerService } from '@app/services/player.service';
import { PlayerDialComponent } from '../player-dial/player-dial.component';

@Component({
    selector: 'app-player-controls',
    imports: [
        MatIcon,
        MatIconButton,
        PlayerDialComponent
    ],
    templateUrl: './player-controls.component.html',
    styleUrl: './player-controls.component.scss',
    })
export class PlayerControlsComponent {
    protected readonly player = inject(PlayerService);
}
