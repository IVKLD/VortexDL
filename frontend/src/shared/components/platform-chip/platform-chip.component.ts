import {Component, input} from '@angular/core';
import {MatIcon} from '@angular/material/icon';

export enum SupportedPlatform {
    YouTube = 'youtube',
    SoundCloud = 'soundcloud',
}

@Component({
    selector: 'app-platform-chip',
    imports: [MatIcon],
    templateUrl: './platform-chip.component.html',
    styleUrl: './platform-chip.component.scss'
})
export class PlatformChipComponent {
    public readonly platform = input.required<SupportedPlatform>();
    protected readonly SupportedPlatform = SupportedPlatform;
}


