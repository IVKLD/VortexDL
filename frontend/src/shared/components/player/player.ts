import {ChangeDetectionStrategy, Component, computed, inject} from '@angular/core';
import {MatFabButton, MatIconButton} from '@angular/material/button';
import {MatIcon} from '@angular/material/icon';
import {MatSlider, MatSliderThumb} from '@angular/material/slider';
import {PlayerService} from '@app/services/player.service';

@Component({
    selector: 'app-player',
    imports: [
        MatIcon,
        MatSlider,
        MatSliderThumb,
        MatFabButton,
        MatIconButton
    ],
    templateUrl: './player.html',
    styleUrl: './player.scss',
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PlayerComponent {
    protected readonly player = inject(PlayerService);

    protected readonly currentTimeFormatted = computed(() => this.formatTime(this.player.progress()));
    protected readonly durationFormatted = computed(() => this.formatTime(this.player.duration()));

    protected readonly playIcon = computed(() =>
        this.player.isPlaying() ? 'pause' : 'play_arrow'
    );

    protected readonly volumeIcon = computed(() => {
        const vol = this.player.volume();
        if (vol === 0) return 'volume_off';
        if (vol < 0.5) return 'volume_down';
        return 'volume_up';
    });

    protected onVolumeInput(value: string): void {
        this.player.setVolume(+value);
    }

    protected onSeekInput(value: string): void {
        this.player.seek(+value);
    }

    private formatTime(seconds: number): string {
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    }
}
