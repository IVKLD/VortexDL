import { Injectable, signal } from '@angular/core';
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
}
