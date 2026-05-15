import {computed, Injectable, signal} from '@angular/core';

@Injectable({
    providedIn: 'root',
})
export class LoadingService {
    private readonly _loadingCount = signal(0);

    public readonly isLoading = computed(() => this._loadingCount() > 0);

    public start(): void {
        this._loadingCount.update(count => count + 1);
    }

    public stop(): void {
        this._loadingCount.update(count => Math.max(0, count - 1));
    }
}
