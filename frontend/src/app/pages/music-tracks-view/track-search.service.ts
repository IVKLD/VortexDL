import { computed, Injectable, signal } from '@angular/core';
import { debounce, form } from '@angular/forms/signals';
import { MusicTrack } from '@shared/models/music-track.model';
import Fuse from 'fuse.js';

@Injectable({ providedIn: 'root' })
export class TrackSearch {
    public readonly query = signal<string>('');

    public readonly form = form(this.query, (p) => {
        debounce(p, 200);
    });

    public readonly isSearching = computed(() => this.form().value() !== this.query());
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

        return fuse.search(q).map((r) => r.item);
    }

    public setQuery(query: string): void {
        this.query.set(query);
    }

    public clear(): void {
        this.query.set('');
    }
}
