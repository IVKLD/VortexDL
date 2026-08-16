import { computed, inject, Injectable, signal } from '@angular/core';
import { MusicTrack } from '@shared/models/music-track.model';
import { Observable, tap } from 'rxjs';
import Fuse from 'fuse.js';
import { MusicTracksViewService } from '@app/pages/music-tracks-view/music-tracks-view.service';
import { form, debounce } from '@angular/forms/signals';

export enum MusicSortOption {
    NAME_ASC = 'name-asc',
    NAME_DESC = 'name-desc',
    DATE_DESC = 'date-desc',
    DATE_ASC = 'date-asc',
}

@Injectable({ providedIn: 'root' })
export class MusicTracksViewState {
    private readonly _api = inject(MusicTracksViewService);

    private readonly _isLoading = signal<boolean>(true);
    private readonly _tracks = signal<MusicTrack[]>([]);
    private readonly _sortOption = signal<MusicSortOption>(MusicSortOption.DATE_DESC);
    private readonly _selectedIds = signal<Set<number>>(new Set());

    private readonly _fuse = computed(() =>
        new Fuse(this._tracks(), {
            keys: [
                { name: 'title', weight: 1 },
                { name: 'artist', weight: 0.7 },
            ],
            threshold: 0.3,
            distance: 100,
            ignoreLocation: true,
        })
    );

    public readonly searchQuery = signal<string>('');

    public readonly searchForm = form(this.searchQuery, (p) => {
        debounce(p, 200);
    });

    public readonly isSearching = computed(() => {
        return this.searchForm().value() !== this.searchQuery();
    });

    public readonly isLoading = this._isLoading.asReadonly();
    public readonly tracks = this._tracks.asReadonly();
    public readonly sortOption = this._sortOption.asReadonly();
    public readonly selectedIds = this._selectedIds.asReadonly();

    public readonly hasSelection = computed(() => this._selectedIds().size > 0);
    public readonly selectedTracks = computed(() => this._tracks().filter(t => this._selectedIds().has(t.id)));

    public readonly sortedTracks = computed(() => {
        const query = this.searchQuery().trim();
        if (query) {
            return this._fuse().search(query).map(r => r.item);
        }

        const [sort, order] = this._sortOption().split('-');
        return [...this._tracks()].sort((a, b) => {
            const valA = sort === 'name' ? a.title.toLowerCase() : a.createdAt;
            const valB = sort === 'name' ? b.title.toLowerCase() : b.createdAt;
            const cmp = typeof valA === 'string' ? valA.localeCompare(valB as string) : (valB as number) - (valA as number);
            return order === 'desc' ? -cmp : cmp;
        });
    });

    constructor() {
        this.loadTracks();
    }

    private loadTracks(): void {
        this.startLoading();
        this._api.getAll().subscribe({ next: tracks => this.setTracks = tracks });
    }

    public set setTracks(value: MusicTrack[]) {
        this._tracks.set(value);
        this._isLoading.set(false);
    }

    public addTrack(track: MusicTrack): void {
        this._tracks.update(data => [...data, track]);
    }

    public removeTrack(id: number): void {
        this._tracks.update(data => data.filter(t => t.id !== id));
        this._selectedIds.update(ids => (ids.delete(id), new Set(ids)));
    }

    public selectTrack(track: MusicTrack): void {
        this._selectedIds.update(ids => ids.has(track.id) ? ids : new Set(ids).add(track.id));
    }

    public deselectTrack(track: MusicTrack): void {
        this._selectedIds.update(ids => ids.has(track.id) ? (ids.delete(track.id), new Set(ids)) : ids);
    }

    public toggleSelect(track: MusicTrack): void {
        this._selectedIds.update(ids => {
            const next = new Set(ids);
            return next.delete(track.id) ? next : next.add(track.id);
        });
    }

    public clearSelection(): void {
        this._selectedIds.set(new Set());
    }

    public startLoading(): void {
        this._isLoading.set(true);
    }

    public refresh(): Observable<MusicTrack[]> {
        this.startLoading();
        return this._api.getAll().pipe(tap(tracks => this.setTracks = tracks));
    }

    public setSortOption(option: MusicSortOption): void {
        this._sortOption.set(option);
    }

    public setSearchQuery(query: string): void {
        this.searchQuery.set(query);
    }
}
