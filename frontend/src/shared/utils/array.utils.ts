export function ensureStringArray(value: string[] | string | null | undefined): string[] {
    if (!value) return [];
    if (typeof value === 'string') return [value];
    if (Array.isArray(value)) {
        return value.filter((item): item is string => typeof item === 'string');
    }
    return [];
}

export function shuffleArray<T>(array: T[]): T[] {
    const result = [...array];
    for (let i = result.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        const temp = result[i]!;
        result[i] = result[j]!;
        result[j] = temp;
    }
    return result;
}


