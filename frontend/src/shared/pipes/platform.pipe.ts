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

        if (lower.includes('youtube') || lower.includes('youtu.be')) {
            return SupportedPlatform.YouTube;
        }
        if (lower.includes('soundcloud')) {
            return SupportedPlatform.SoundCloud;
        }

        return false;
    }
}
