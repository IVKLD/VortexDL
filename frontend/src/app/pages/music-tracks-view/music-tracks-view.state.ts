import { computed, effect, inject, Injectable, signal, untracked } from '@angular/core';
import { MusicTrack } from '@shared/models/music-track.model';
import { Observable, tap } from 'rxjs';
import Fuse from 'fuse.js';
import { MusicTracksViewService } from '@app/pages/music-tracks-view/music-tracks-view.service';

export enum MusicSortOption {
    POSITION_ASC = 'position-asc',
    NAME_ASC = 'name-asc',
    NAME_DESC = 'name-desc',
    DATE_DESC = 'date-desc',
    DATE_ASC = 'date-asc',
}

@Injectable({ providedIn: 'root' })
export class MusicTracksViewState {
    constructor() {
        effect(() => {
            const option = this.sortOption();
            untracked(() => {
                this.startLoading();
                const [sort, order] = option.split('-');
                this._api.getAll(sort, order).subscribe({ next: tracks => this.setTracks = tracks });
            });
        });
    }

    private readonly _api = inject(MusicTracksViewService);

    private readonly _isLoading = signal<boolean>(true);
    public readonly isLoading = this._isLoading.asReadonly();

    private readonly _tracks = signal<MusicTrack[]>([]);
    public readonly countMusicTracks = computed(() => this._tracks().length);

    private readonly _sortOption = signal<MusicSortOption>(MusicSortOption.POSITION_ASC);
    public readonly sortOption = this._sortOption.asReadonly();

    private readonly _searchQuery = signal<string>('');
    public readonly searchQuery = this._searchQuery.asReadonly();

    private readonly _selectedIds = signal<Set<number>>(new Set());
    public readonly selectedIds = this._selectedIds.asReadonly();

    public readonly hasSelection = computed(() => this._selectedIds().size > 0);
    public readonly selectedTracks = computed(() => this._tracks().filter(t => this._selectedIds().has(t.id)));

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

    public readonly sortedTracks = computed(() => {
        const query = this._searchQuery().trim();
        if (query) {
            return this._fuse().search(query).map(r => r.item);
        }

        const [sort, order] = this._sortOption().split('-');
        return [...this._tracks()].sort((a, b) => {
            const valA = sort === 'name' ? a.title.toLowerCase() : sort === 'date' ? a.createdAt : (a.position ?? 4294967295);
            const valB = sort === 'name' ? b.title.toLowerCase() : sort === 'date' ? b.createdAt : (b.position ?? 4294967295);
            const cmp = typeof valA === 'string' ? valA.localeCompare(valB as string) : (valA as number) - (valB as number);
            return order === 'desc' ? -cmp : cmp;
        });
    });

    public set setTracks(value: MusicTrack[]) {
        this._tracks.set(value);
        this._isLoading.set(false);
    }

    public addTrack(track: MusicTrack): void {
        this._tracks.update(data => [...data, track]);
    }

    public removeTrack(track: MusicTrack): void {
        this._tracks.update(data => data.filter(t => t.id !== track.id));
        this._selectedIds.update(ids => (ids.delete(track.id), new Set(ids)));
    }

    public toggleSelect(track: MusicTrack): void {
        this._selectedIds.update(ids => {
            const next = new Set(ids);
            next.has(track.id) ? next.delete(track.id) : next.add(track.id);
            return next;
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
        const [sort, order] = this.sortOption().split('-');
        return this._api.getAll(sort, order).pipe(tap(tracks => this.setTracks = tracks));
    }

    public setSortOption(option: MusicSortOption): void {
        this._sortOption.set(option);
    }

    public setSearchQuery(query: string): void {
        this._searchQuery.set(query);
    }
}
