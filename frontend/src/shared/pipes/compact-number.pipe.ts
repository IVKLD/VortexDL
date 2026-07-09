import { Pipe, PipeTransform } from '@angular/core';

/**
 * Transforms numbers into a compact human-readable format.
 *
 * Examples:
 * - null / undefined -> '—'
 * - 950              -> '950'
 * - 1000             -> '1K'
 * - 1543             -> '1.5K'
 * - 1200000          -> '1.2M'
 */
@Pipe({
    name: 'compactNumber',
    standalone: true
})
export class CompactNumberPipe implements PipeTransform {
    transform(count: number | null | undefined): string {
        if (count == null) return '—';
        return new Intl.NumberFormat('en', {
            notation: 'compact',
            maximumFractionDigits: 1
        }).format(count);
    }
}
