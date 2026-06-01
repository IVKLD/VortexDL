import {ChangeDetectionStrategy, Component, computed, inject} from '@angular/core';
import {MatIconButton} from '@angular/material/button';
import {MatIcon} from '@angular/material/icon';
import {MatSlider, MatSliderThumb} from '@angular/material/slider';
import {PlayerService} from '@app/services/player.service';

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
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PlayerVolumeComponent {
    protected readonly player = inject(PlayerService);

    protected readonly volumeIcon = computed(() => {
        const vol = this.player.volume();
        if (vol === 0) return 'volume_off';
        if (vol < 0.5) return 'volume_down';
        return 'volume_up';
    });

    protected onVolumeInput(value: string): void {
        this.player.setVolume(+value);
    }

    private prevVolume = this.player.volume();

    protected mute(): void {
        const currentVol = this.player.volume();
        if (currentVol > 0) {
            this.prevVolume = currentVol;
            this.player.setVolume(0);
        } else {
            this.player.setVolume(this.prevVolume);
        }
    }
}
