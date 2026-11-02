import { computed, inject, Injectable, signal } from '@angular/core';
import { MusicTracksService } from '@app/pages/music-tracks-view/music-tracks.service';
import { LoadingService } from '@app/services/loading.service';
import { MusicTrack } from '@shared/models/music-track.model';
import { catchError, Observable, of, switchMap, tap } from 'rxjs';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { MusicSortOption } from './music-tracks-view.model';
import { TrackSearch } from './track-search.service';
import { TrackSelection } from './track-selection.service';

export { MusicSortOption } from './music-tracks-view.model';
export { TrackSearch } from './track-search.service';
export { TrackSelection } from './track-selection.service';

@Injectable({ providedIn: 'root' })
export class MusicTracksViewState {
    private readonly _api = inject(MusicTracksService);
    public readonly selection = inject(TrackSelection);
    public readonly search = inject(TrackSearch);

    private readonly _tracks = signal<MusicTrack[]>([]);
    private readonly _sortOption = signal<MusicSortOption>(MusicSortOption.DATE_DESC);

    public readonly isLoading = inject(LoadingService).isLoading;
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
        this._api
            .indexing()
            .pipe(
                catchError(() => of(undefined)),
                switchMap(() => this._api.getAll()),
                takeUntilDestroyed(),
            )
            .subscribe({ next: this._tracks.set });
    }

    public isTrackLocal(id: number): boolean {
        return this._tracks().some((t) => t.id === id);
    }

    public addTrack(track: MusicTrack): void {
        this._tracks.update((data) => [...data, track]);
    }

    public removeTracks(ids: number[]): void {
        this._tracks.update((data) => data.filter((t) => !ids.includes(t.id)));
        this.selection.remove(ids);
    }

    public refresh(): Observable<MusicTrack[]> {
        return this._api.getAll().pipe(tap(this._tracks.set));
    }

    public setSortOption(option: MusicSortOption): void {
        this._sortOption.set(option);
    }
}
