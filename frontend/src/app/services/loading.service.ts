import { computed, Injectable, signal } from '@angular/core';

@Injectable({ providedIn: 'platform' })
export class LoadingService {
    private readonly _count = signal(0);

    public readonly isLoading = computed(() => this._count() > 0);

    public start() {
        this._count.update((c) => c + 1);
    }

    public stop() {
        this._count.update((c) => Math.max(0, c - 1));
    }
}
