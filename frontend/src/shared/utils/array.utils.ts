export function ensureStringArray(value: string[] | string | null | undefined): string[] {
    if (!value) return [];
    if (typeof value === 'string') return [value];
    if (Array.isArray(value)) {
        return value.filter((item): item is string => typeof item === 'string');
    }
    return [];
}

