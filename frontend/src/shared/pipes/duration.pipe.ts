import { Pipe, PipeTransform } from '@angular/core';

export const formatDurationSeconds = (totalSeconds: number): string => {
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = Math.floor(totalSeconds % 60);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
};

@Pipe({ name: 'duration', standalone: true })
export class DurationPipe implements PipeTransform {
    transform(ms: number | null): string {
        if (ms === null || ms === undefined) return '—';
        return formatDurationSeconds(Math.floor(ms / 1000));
    }
}
