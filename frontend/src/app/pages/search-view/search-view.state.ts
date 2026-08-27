import { Injectable, inject, signal } from '@angular/core';
import { SearchViewService } from './search-view.service';
import { SearchDurationFilter, SearchProvider, SearchSettingsModel, SearchTrackItemRdo } from './models/search-view.model';
import { finalize } from 'rxjs';
import { NotificationService } from '@app/services/notification.service';
import { SearchHistoryService } from './search-history.service';
import { SettingsState } from '@app/pages/settings-view/settings.state';

const SEARCH_SETTINGS_KEY = 'vortexdl_search_settings';

@Injectable({ providedIn: 'root' })
export class SearchViewState {
    private readonly _api = inject(SearchViewService);
    private readonly _notification = inject(NotificationService);
    private readonly _history = inject(SearchHistoryService);
    private readonly _settingsState = inject(SettingsState);

    public readonly query = signal<string>('');
    public readonly provider = signal<SearchProvider>(SearchProvider.SoundCloud);
    public readonly duration = signal<SearchDurationFilter>(SearchDurationFilter.Any);

    public readonly hasSearched = signal<boolean>(false);
    public readonly showHistory = signal<boolean>(false);
    public readonly rawTracks = signal<SearchTrackItemRdo[]>([]);
    public readonly loading = signal(false);
    public readonly hasMore = signal(false);

    public readonly results = this.rawTracks.asReadonly();

    constructor() {
        this.loadSavedSettings();
    }

    private loadSavedSettings(): void {
        try {
            const raw = localStorage.getItem(SEARCH_SETTINGS_KEY);
            if (raw) {
                const parsed = JSON.parse(raw) as SearchSettingsModel;
                if (parsed.provider) this.provider.set(parsed.provider);
                if (parsed.duration) this.duration.set(parsed.duration);
            }
        } catch {
            // ignore
        }
    }

    private saveSettings(): void {
        localStorage.setItem(
            SEARCH_SETTINGS_KEY,
            JSON.stringify({ provider: this.provider(), duration: this.duration() })
        );
    }

    public setProvider(provider: SearchProvider): void {
        if (this.provider() === provider) return;
        this.provider.set(provider);
        this.saveSettings();
        if (this.hasSearched() && this.query()) {
            this.search(this.query());
        }
    }

    public setDuration(duration: SearchDurationFilter): void {
        if (this.duration() === duration) return;
        this.duration.set(duration);
        this.saveSettings();
        if (this.hasSearched() && this.query()) {
            this.search(this.query());
        }
    }

    public clear(): void {
        this.query.set('');
        this.hasSearched.set(false);
        this.showHistory.set(false);
        this.rawTracks.set([]);
        this.loading.set(false);
        this.hasMore.set(false);
    }

    public search(query: string): void {
        const trimmed = query.trim();
        if (!trimmed) {
            this.clear();
            return;
        }

        this.query.set(trimmed);
        this.hasSearched.set(true);
        this.showHistory.set(false);
        this._history.add(trimmed);
        this.rawTracks.set([]);

        this.loading.set(true);
        const limit = this._settingsState.settingsModel().system?.limitPerPage ?? 20;
        const dur = this.provider() === SearchProvider.YouTube ? this.duration() : undefined;
        this._api
            .searchTracks(trimmed, limit, 0, this.provider(), dur)
            .pipe(finalize(() => this.loading.set(false)))
            .subscribe({
                next: (res) => {
                    this.rawTracks.set(res.tracks);
                    this.hasMore.set(res.hasMore);
                },
                error: () => this._notification.error('Search failed. Try again.'),
            });
    }

    public loadMore(): void {
        const q = this.query().trim();
        if (!q || this.loading() || !this.hasMore() || this.rawTracks().length === 0) return;

        this.loading.set(true);
        const limit = this._settingsState.settingsModel().system?.limitPerPage ?? 20;
        const dur = this.provider() === SearchProvider.YouTube ? this.duration() : undefined;
        this._api
            .searchTracks(q, limit, this.rawTracks().length, this.provider(), dur)
            .pipe(finalize(() => this.loading.set(false)))
            .subscribe({
                next: (res) => {
                    const existingIds = new Set(this.rawTracks().map((t) => t.id));
                    const uniqueNewTracks = res.tracks.filter((t) => !existingIds.has(t.id));
                    this.rawTracks.update((prev) => [...prev, ...uniqueNewTracks]);
                    this.hasMore.set(res.hasMore);
                },
            });
    }
}
