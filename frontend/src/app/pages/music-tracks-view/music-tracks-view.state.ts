import { computed, Injectable, signal } from "@angular/core";
import { Track } from "./music-tracks-view.service";
import Fuse from 'fuse.js';

export enum MusicSortOption {
    NAME_ASC = 'name-asc',
    NAME_DESC = 'name-desc',
    DATE = 'date'
}

@Injectable({
    providedIn: 'root',
})
export class MusicTracksViewState {
    private readonly _tracks = signal<Track[]>([]);
    private readonly _sortOption = signal<MusicSortOption>(MusicSortOption.DATE);
    private readonly _searchQuery = signal<string>('');

    public readonly sortOption = this._sortOption.asReadonly();
    public readonly searchQuery = this._searchQuery.asReadonly();

    private readonly _fuse = computed(() => new Fuse(this._tracks(), {
        keys: [
            { name: 'filename', weight: 1 },
            { name: 'album', weight: 0.3 }
        ],
        threshold: 0.3,
        distance: 100,
        ignoreLocation: true
    }));

    public readonly sortedTracks = computed(() => {
        let tracks = [...this._tracks()];
        const query = this._searchQuery().trim();

        if (query) {
            tracks = this._fuse().search(query).map(result => result.item);
        }

        const option = this._sortOption();
        
        switch (option) {
            case MusicSortOption.NAME_ASC:
                return tracks.sort((a, b) => a.filename.localeCompare(b.filename));
            case MusicSortOption.NAME_DESC:
                return tracks.sort((a, b) => b.filename.localeCompare(a.filename));
            case MusicSortOption.DATE:
                return tracks.sort((a, b) => b.created_at - a.created_at);
            default:
                return tracks;
        }
    });

    public set setTracks(value: Track[]) {
        this._tracks.set(this.normalizeTracks(value));
    }

    public setSortOption(option: MusicSortOption) {
        this._sortOption.set(option);
    }

    public setSearchQuery(query: string) {
        this._searchQuery.set(query);
    }

    public addTrack(music: Track) {
        this._tracks.update(data => [...data, this.normalizeTrack(music)]);
    }

    public removeTrack(music: Track) {
        this._tracks.update(data => data.filter(item => item.id !== music.id));
    }

    /** Clean up and normalize track list */
    private normalizeTracks(tracks: Track[]): Track[] {
        return tracks.map(t => this.normalizeTrack(t));
    }

    /** Remove redundant spaces for consistent sorting and searching */
    private normalizeTrack(track: Track): Track {
        return {
            ...track,
            filename: track.filename.trim(),
            album: track.album?.trim()
        };
    }
}