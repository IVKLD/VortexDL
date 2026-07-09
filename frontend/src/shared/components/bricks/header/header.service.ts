import { DestroyRef, inject, Injectable, signal } from '@angular/core';
import { HeaderConfig, HeaderFeature, HeaderSearchBind, HeaderSortBind } from './header.types';

@Injectable({
    providedIn: 'root'
})
export class HeaderService {
    public readonly config = signal<HeaderConfig | undefined>(undefined);
    public readonly searchBind = signal<HeaderSearchBind | null>(null);
    public readonly sortBind = signal<HeaderSortBind<unknown> | null>(null);

    public hasFeature(feature: HeaderFeature): boolean {
        return this.config()?.features.includes(feature) ?? false;
    }

    public bindSearch(bind: HeaderSearchBind, destroyRef?: DestroyRef): () => void {
        this.searchBind.set(bind);
        const cleanup = () => {
            if (this.searchBind() === bind) {
                this.searchBind.set(null);
            }
        };

        const resolvedDestroyRef = destroyRef || inject(DestroyRef, { optional: true });
        if (resolvedDestroyRef) {
            resolvedDestroyRef.onDestroy(cleanup);
        }

        return cleanup;
    }

    public bindSort<T>(bind: HeaderSortBind<T>, destroyRef?: DestroyRef): () => void {
        const unboundBind = bind as unknown as HeaderSortBind<unknown>;
        this.sortBind.set(unboundBind);
        const cleanup = () => {
            if (this.sortBind() === unboundBind) {
                this.sortBind.set(null);
            }
        };

        const resolvedDestroyRef = destroyRef || inject(DestroyRef, { optional: true });
        if (resolvedDestroyRef) {
            resolvedDestroyRef.onDestroy(cleanup);
        }

        return cleanup;
    }
}









