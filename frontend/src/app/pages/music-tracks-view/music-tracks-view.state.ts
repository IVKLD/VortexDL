import {computed, inject, Injectable, signal} from '@angular/core';
import {MusicTrack} from '@shared/models/music-track.model';
import {Observable, tap} from 'rxjs';
import Fuse from 'fuse.js';
import {MusicTracksService} from '@app/pages/music-tracks-view/music-tracks.service';
import {debounce, form} from '@angular/forms/signals';

export enum MusicSortOption {
    NAME_ASC = 'name-asc',
    NAME_DESC = 'name-desc',
    DATE_DESC = 'date-desc',
    DATE_ASC = 'date-asc',
}

@Injectable({ providedIn: 'root' })
export class TrackSelection {
    private readonly _ids = signal<number[]>([]);

    public readonly ids = this._ids.asReadonly();
    public readonly count = computed(() => this._ids().length);
    public readonly hasSelection = computed(() => this._ids().length > 0);

    public has(id: number): boolean {
        return this._ids().includes(id);
    }

    public select(id: number): void {
        this._ids.update(ids => ids.includes(id) ? ids : [...ids, id]);
    }

    public deselect(id: number): void {
        this._ids.update(ids => ids.filter(i => i !== id));
    }

    public toggle(id: number): void {
        this._ids.update(ids => ids.includes(id) ? ids.filter(i => i !== id) : [...ids, id]);
    }

    public remove(ids: number[]): void {
        this._ids.update(current => current.filter(id => !ids.includes(id)));
    }

    public clear(): void {
        this._ids.set([]);
    }
}

@Injectable({ providedIn: 'root' })
export class TrackSearch {
    public readonly query = signal<string>('');

    public readonly form = form(this.query, (p) => {
        debounce(p, 200);
    });

    public readonly isSearching = computed(() => {
        return this.form().value() !== this.query();
    });

    public readonly hasQuery = computed(() => this.query().trim().length > 0);
    public readonly isEmpty = computed(() => this.query().trim().length === 0);

    public filter(tracks: MusicTrack[]): MusicTrack[] {
        const q = this.query().trim();
        if (!q) {
            return tracks;
        }

        const fuse = new Fuse(tracks, {
            keys: [
                { name: 'title', weight: 1 },
                { name: 'artist', weight: 0.7 },
            ],
            threshold: 0.3,
            distance: 100,
            ignoreLocation: true,
        });

        return fuse.search(q).map(r => r.item);
    }

    public setQuery(query: string): void {
        this.query.set(query);
    }

    public clear(): void {
        this.query.set('');
    }
}

@Injectable({ providedIn: 'root' })
export class MusicTracksViewState {
    private readonly _api = inject(MusicTracksService);
    public readonly selection = inject(TrackSelection);
    public readonly search = inject(TrackSearch);

    private readonly _tracks = signal<MusicTrack[]>([]);
    private readonly _sortOption = signal<MusicSortOption>(MusicSortOption.DATE_DESC);

    public readonly isLoading = this._api.isLoading.asReadonly();
    public readonly tracks = this._tracks.asReadonly();
    public readonly sortOption = this._sortOption.asReadonly();

    public readonly sortedTracks = computed(() => {
        if (this.search.hasQuery()) {
            return this.search.filter(this._tracks());
        }

        const [sort, order] = this._sortOption().split('-');
        return [...this._tracks()].sort((a, b) => {
            const valA = sort === 'name' ? a.title.toLowerCase() : a.createdAt;
            const valB = sort === 'name' ? b.title.toLowerCase() : b.createdAt;
            const cmp = typeof valA === 'string' ? valA.localeCompare(valB as string) : (valA as number) - (valB as number);
            return order === 'desc' ? -cmp : cmp;
        });
    });

    constructor() {
        this.loadTracks();
    }

    private loadTracks(): void {
        this._api.indexing().subscribe();
        this._api.getAll().subscribe({ next: tracks => this._tracks.set(tracks) });
    }

    public isTrackLocal(id: number): boolean {
        return this._tracks().some(t => t.id === id);
    }

    public addTrack(track: MusicTrack): void {
        this._tracks.update(data => [...data, track]);
    }

    public removeTracks(ids: number[]): void {
        this._tracks.update(data => data.filter(t => !ids.includes(t.id)));
        this.selection.remove(ids);
    }

    public refresh(): Observable<MusicTrack[]> {
        return this._api.getAll().pipe(tap(tracks => this._tracks.set(tracks)));
    }

    public setSortOption(option: MusicSortOption): void {
        this._sortOption.set(option);
    }
}
