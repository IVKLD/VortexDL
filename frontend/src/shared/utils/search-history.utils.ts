const HISTORY_KEY = 'search_history';
const MAX_ITEMS = 10;

export function loadSearchHistory(): string[] {
    try {
        const stored = localStorage.getItem(HISTORY_KEY);
        return stored ? JSON.parse(stored) : [];
    } catch {
        return [];
    }
}

export function addToSearchHistory(current: string[], item: string): string[] {
    const trimmed = item.trim();
    if (!trimmed) return current;

    const filtered = current.filter(i => i.toLowerCase() !== trimmed.toLowerCase());
    const updated = [trimmed, ...filtered].slice(0, MAX_ITEMS);
    
    try {
        localStorage.setItem(HISTORY_KEY, JSON.stringify(updated));
    } catch (_err) {
        // Ignore localStorage error
    }

    return updated;
}

export function removeFromSearchHistory(current: string[], item: string): string[] {
    const updated = current.filter(i => i !== item);
    try {
        localStorage.setItem(HISTORY_KEY, JSON.stringify(updated));
    } catch (_err) {
        // Ignore localStorage error
    }
    return updated;
}

export function clearSearchHistory(): string[] {
    try {
        localStorage.removeItem(HISTORY_KEY);
    } catch (_err) {
        // Ignore localStorage error
    }
    return [];
}
