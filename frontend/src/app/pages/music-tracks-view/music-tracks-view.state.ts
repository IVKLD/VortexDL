import {computed, effect, inject, Injectable, signal, untracked} from '@angular/core';
import {Track} from '@shared/models/track.model';
import Fuse from 'fuse.js';
import {MusicTracksViewService} from "@app/pages/music-tracks-view/music-tracks-view.service";

export enum MusicSortOption {
    POSITION_ASC = 'position-asc',
    NAME_ASC = 'name-asc',
    NAME_DESC = 'name-desc',
    DATE_DESC = 'date-desc',
    DATE_ASC = 'date-asc',
}

@Injectable({
    providedIn: 'root',
})
export class MusicTracksViewState {
    private readonly _api = inject(MusicTracksViewService);

    constructor() {
        effect(() => {
            const option = this.sortOption();
            untracked(() => {
                this.startLoading();
                const [sort, order] = option.split('-');

                this._api.getAll(sort, order)
                    .subscribe({
                        next: tracks => this.setTracks = tracks
                    });
            });
        });
    }

    private readonly _isLoading = signal<boolean>(true);
    public readonly isLoading = this._isLoading.asReadonly();

    private readonly _tracks = signal<Track[]>([]);
    public readonly count_tracks = computed(() => this._tracks().length);

    private readonly _sortOption = signal<MusicSortOption>(MusicSortOption.POSITION_ASC);
    public readonly sortOption = this._sortOption.asReadonly();

    private readonly _searchQuery = signal<string>('');
    public readonly searchQuery = this._searchQuery.asReadonly();

    private readonly _fuse = computed(() =>
        new Fuse(this._tracks(), {
            keys: [
                {name: 'title', weight: 1},
                {name: 'artist', weight: 0.7},
            ],
            threshold: 0.3,
            distance: 100,
            ignoreLocation: true,
        })
    );

    public readonly sortedTracks = computed(() => {
        const query = this._searchQuery().trim();
        if (query) {
            return this._fuse().search(query).map(result => result.item);
        }

        const option = this._sortOption();
        const [sort, order] = option.split('-');

        return [...this._tracks()].sort((a, b) => {
            const valA = sort === 'name' ? (a.title).toLowerCase() : sort === 'date' ? a.createdAt : (a.position ?? 4294967295);
            const valB = sort === 'name' ? (b.title).toLowerCase() : sort === 'date' ? b.createdAt : (b.position ?? 4294967295);
            
            if (typeof valA === 'string' && typeof valB === 'string') {
                const cmp = valA.localeCompare(valB);
                return order === 'desc' ? -cmp : cmp;
            }
            if (typeof valA === 'number' && typeof valB === 'number') {
                const cmp = valA - valB;
                return order === 'desc' ? -cmp : cmp;
            }
            return 0;
        });
    });

    public set setTracks(value: Track[]) {
        this._tracks.set(value);
        this._isLoading.set(false);
    }

    public addTrack(music: Track) {
        this._tracks.update(data => [...data, music]);
    }

    public removeTrack(music: Track) {
        this._tracks.update(data => data.filter(item => item.id !== music.id));
    }

    public startLoading() {
        this._isLoading.set(true);
    }

    public setSortOption(option: MusicSortOption) {
        this._sortOption.set(option);
    }

    public setSearchQuery(query: string) {
        this._searchQuery.set(query);
    }
}

