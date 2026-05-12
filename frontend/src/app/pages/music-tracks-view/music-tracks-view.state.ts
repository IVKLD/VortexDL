import { computed, Injectable, signal } from '@angular/core';
import { Track } from '@shared/models/track.model';
import Fuse from 'fuse.js';

export enum MusicSortOption {
    NAME_ASC = 'name-asc',
    NAME_DESC = 'name-desc',
    DATE = 'date',
}

@Injectable({
    providedIn: 'root',
})
export class MusicTracksViewState {
    private readonly _isLoading = signal<boolean>(true);
    public readonly isLoading = this._isLoading.asReadonly();
    private readonly _tracks = signal<Track[]>([]);
    private readonly _sortOption = signal<MusicSortOption>(MusicSortOption.DATE);
    public readonly sortOption = this._sortOption.asReadonly();
    private readonly _searchQuery = signal<string>('');
    public readonly searchQuery = this._searchQuery.asReadonly();

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
        let tracks = [...this._tracks()];
        const query = this._searchQuery().trim();

        if (query) {
            tracks = this._fuse()
                .search(query)
                .map(result => result.item);
        }

        const option = this._sortOption();

        switch (option) {
            case MusicSortOption.NAME_ASC:
                return tracks.sort((a, b) => a.title.localeCompare(b.title));
            case MusicSortOption.NAME_DESC:
                return tracks.sort((a, b) => b.title.localeCompare(a.title));
            case MusicSortOption.DATE:
                return tracks.sort((a, b) => b.createdAt - a.createdAt);
            default:
                return tracks;
        }
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

