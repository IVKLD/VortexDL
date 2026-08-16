import { Component, computed, input } from '@angular/core';
import { MatIcon } from '@angular/material/icon';

export type SupportedPlatform = 'youtube' | 'soundcloud' | 'auto';

@Component({
    selector: 'app-platform-chip',
    standalone: true,
    imports: [MatIcon],
    templateUrl: './platform-chip.component.html',
    styleUrl: './platform-chip.component.scss'
})
export class PlatformChipComponent {
    public readonly url = input<string | null | undefined>(null);
    public readonly platform = input<SupportedPlatform | string | null | undefined>(null);
    public readonly showAuto = input<boolean>(false);

    public readonly detectedPlatform = computed<SupportedPlatform>(() => {
        const directPlatform = this.platform()?.toLowerCase();
        if (directPlatform === 'youtube') return 'youtube';
        if (directPlatform === 'soundcloud') return 'soundcloud';

        const rawUrl = (this.url() || '').trim().toLowerCase();
        if (!rawUrl) return 'auto';

        if (rawUrl.includes('youtube.com') || rawUrl.includes('youtu.be')) {
            return 'youtube';
        }
        if (rawUrl.includes('soundcloud.com')) {
            return 'soundcloud';
        }

        return 'auto';
    });
}
