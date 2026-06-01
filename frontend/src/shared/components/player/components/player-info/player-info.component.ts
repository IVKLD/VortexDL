import {ChangeDetectionStrategy, Component, input} from '@angular/core';
import {NgOptimizedImage} from "@angular/common";
import {MatIcon} from '@angular/material/icon';
import {Track} from "@shared/models/track.model";

@Component({
    selector: 'app-player-info',
    imports: [
        MatIcon,
        NgOptimizedImage
    ],
    templateUrl: './player-info.component.html',
    styleUrl: './player-info.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PlayerInfoComponent {
    public readonly data = input.required<Track>();
}
