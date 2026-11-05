export interface StorageHistoryHelper {
    load(): string[];
    add(current: string[], item: string): string[];
    remove(current: string[], item: string): string[];
    clear(): string[];
}

export const createStorageHistory = (key: string, maxItems = 10): StorageHistoryHelper => ({
    load(): string[] {
        try {
            const stored = localStorage.getItem(key);
            return stored ? JSON.parse(stored) : [];
        } catch {
            return [];
        }
    },
    add(current: string[], item: string): string[] {
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
    remove(current: string[], item: string): string[] {
        const updated = current.filter(i => i !== item);
        try {
            localStorage.setItem(key, JSON.stringify(updated));
        } catch {
            // Ignore localStorage error
        }
        return updated;
    },
    clear(): string[] {
        try {
            localStorage.removeItem(key);
        } catch {
            // Ignore localStorage error
        }
        return [];
    },
});

import { signal, WritableSignal } from '@angular/core';

export abstract class BaseStorageHistoryService {
    private readonly _storage: StorageHistoryHelper;
    public readonly history: WritableSignal<string[]>;

    protected constructor(key: string, maxItems = 10) {
        this._storage = createStorageHistory(key, maxItems);
        this.history = signal<string[]>(this._storage.load());
    }

    public add(item: string): void {
        this.history.update(current => this._storage.add(current, item));
    }

    public remove(item: string): void {
        this.history.update(current => this._storage.remove(current, item));
    }

    public clear(): void {
        this.history.set(this._storage.clear());
    }
}
