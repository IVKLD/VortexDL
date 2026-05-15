import {computed, Injectable, signal} from '@angular/core';
import {Track} from '@shared/models/track.model';
import Fuse from 'fuse.js';

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
    private readonly _isLoading = signal<boolean>(true);
    public readonly isLoading = this._isLoading.asReadonly();

    private readonly _tracks = signal<Track[]>([]);

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
        const tracks = [...this._tracks()];
        const query = this._searchQuery().trim();

        if (query) {
            return this._fuse()
                .search(query)
                .map(result => result.item);
        }

        return tracks;
    });

    public set setTracks(value: Track[]) {
        this._tracks.set(value);
        this._isLoading.set(false);
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

    public addTrack(music: Track) {
        this._tracks.update(data => [...data, music]);
    }

    public removeTrack(music: Track) {
        this._tracks.update(data => data.filter(item => item.id !== music.id));
    }
}

