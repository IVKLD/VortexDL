import { Injectable, signal } from '@angular/core';
import { createStorageHistory } from '@shared/utils/search-history.utils';

@Injectable({ providedIn: 'root' })
export class DownloadHistoryService {
    private readonly _storage = createStorageHistory('vortexdl_download_history', 5);

    public readonly history = signal<string[]>(this._storage.load());

    public add(url: string): void {
        this.history.update(current => this._storage.add(current, url));
    }

    public remove(url: string): void {
        this.history.update(current => this._storage.remove(current, url));
    }

    public clear(): void {
        this.history.set(this._storage.clear());
    }
}
