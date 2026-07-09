import {Component, input} from '@angular/core';
import {NgOptimizedImage} from "@angular/common";
import {MatIcon} from '@angular/material/icon';
import {PlayableTrack} from "@shared/models/music-track.model";

@Component({
    selector: 'app-player-info',
    imports: [
        MatIcon,
        NgOptimizedImage
    ],
    templateUrl: './player-info.component.html',
    styleUrl: './player-info.component.scss',
    })
export class PlayerInfoComponent {
    public readonly data = input.required<PlayableTrack>();
}
