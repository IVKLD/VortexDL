import { Pipe, PipeTransform } from '@angular/core';
import { SupportedPlatform } from '@shared/components/platform-chip/platform-chip.component';

@Pipe({
    name: 'platform',
})
export class PlatformPipe implements PipeTransform {
    transform(input?: string): SupportedPlatform | false {
        const lower = input?.toLowerCase();
        if (!lower) {
            return false;
        }

        if (lower === 'youtube' || lower.includes('youtube.com') || lower.includes('youtu.be')) {
            return SupportedPlatform.YouTube;
        }
        if (lower === 'soundcloud' || lower.includes('soundcloud.com')) {
            return SupportedPlatform.SoundCloud;
        }

        console.error(`[PlatformPipe] Unable to determine platform for input: "${input}"`);
        return false;
    }
}
