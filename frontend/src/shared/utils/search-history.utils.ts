export interface StorageHistoryHelper {
    load(): string[];
    add(current: string[], item: string): string[];
    remove(current: string[], item: string): string[];
    clear(): string[];
}

export const createStorageHistory = (key: string, maxItems = 10): StorageHistoryHelper => ({
    load: (): string[] => {
        try {
            const stored = localStorage.getItem(key);
            return stored ? JSON.parse(stored) : [];
        } catch {
            return [];
        }
    },
    add: (current: string[], item: string): string[] => {
        const trimmed = item.trim();
        if (!trimmed) return current;

        const filtered = current.filter(i => i.toLowerCase() !== trimmed.toLowerCase());
        const updated = [trimmed, ...filtered].slice(0, maxItems);

        try {
            localStorage.setItem(key, JSON.stringify(updated));
        } catch {
            // Ignore localStorage error
        }

        return updated;
    },
    remove: (current: string[], item: string): string[] => {
        const updated = current.filter(i => i !== item);
        try {
            localStorage.setItem(key, JSON.stringify(updated));
        } catch {
            // Ignore localStorage error
        }
        return updated;
    },
    clear: (): string[] => {
        try {
            localStorage.removeItem(key);
        } catch {
            // Ignore localStorage error
        }
        return [];
    },
});

const searchHistory = createStorageHistory('search_history', 10);

export const loadSearchHistory = searchHistory.load;
export const addToSearchHistory = searchHistory.add;
export const removeFromSearchHistory = searchHistory.remove;
export const clearSearchHistory = searchHistory.clear;
