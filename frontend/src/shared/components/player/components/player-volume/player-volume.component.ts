import { Component, computed, input, output } from '@angular/core';
import { MatIconButton } from '@angular/material/button';
import { MatIcon } from '@angular/material/icon';
import { MatSlider, MatSliderThumb } from '@angular/material/slider';

@Component({
    selector: 'app-player-volume',
    imports: [
        MatIcon,
        MatSlider,
        MatSliderThumb,
        MatIconButton
    ],
    templateUrl: './player-volume.component.html',
    styleUrl: './player-volume.component.scss',
})
export class PlayerVolumeComponent {
    public readonly volume = input.required<number>();

    protected readonly volumeIcon = computed(() => {
        const vol = this.volume();
        if (vol === 0) return 'volume_off';
        if (vol < 0.5) return 'volume_down';
        return 'volume_up';
    });

    public readonly volumeChange = output<number>();
    public readonly toggleMute = output<void>();
}
