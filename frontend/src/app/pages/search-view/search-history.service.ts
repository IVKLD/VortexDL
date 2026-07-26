import { Injectable, signal } from '@angular/core';
import {
    loadSearchHistory,
    addToSearchHistory,
    removeFromSearchHistory,
    clearSearchHistory,
} from '@shared/utils/search-history.utils';

@Injectable({ providedIn: 'root' })
export class SearchHistoryService {
    public readonly history = signal<string[]>(loadSearchHistory());

    public add(item: string): void {
        this.history.update(h => addToSearchHistory(h, item));
    }

    public remove(item: string): void {
        this.history.update(h => removeFromSearchHistory(h, item));
    }

    public clear(): void {
        this.history.set(clearSearchHistory());
    }
}
