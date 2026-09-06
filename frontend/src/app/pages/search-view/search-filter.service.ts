import { Injectable } from '@angular/core';
import { SearchTrackItemRdo } from './models/search-track.model';
import { SearchDurationFilter, SearchSortOption } from './models/search-filter.model';

@Injectable({ providedIn: 'root' })
export class SearchFilterService {
    private matchesDuration(durationMs: number | null, filter: SearchDurationFilter): boolean {
        if (durationMs == null) return false;
        const seconds = durationMs / 1000;
        switch (filter) {
            case SearchDurationFilter.Short:
                return seconds < 120;
            case SearchDurationFilter.Medium:
                return seconds >= 120 && seconds <= 600;
            case SearchDurationFilter.Long:
                return seconds > 600 && seconds <= 1800;
            case SearchDurationFilter.Epic:
                return seconds > 1800;
            case SearchDurationFilter.Any:
            default:
                return true;
        }
    }

    private compare(a: SearchTrackItemRdo, b: SearchTrackItemRdo, sort: SearchSortOption): number {
        switch (sort) {
            case SearchSortOption.Popularity:
                return (b.playbackCount ?? 0) - (a.playbackCount ?? 0);
            case SearchSortOption.DurationAsc:
                return (a.duration ?? 0) - (b.duration ?? 0);
            case SearchSortOption.DurationDesc:
                return (b.duration ?? 0) - (a.duration ?? 0);
            case SearchSortOption.TitleAsc:
                return a.title.localeCompare(b.title);
            default:
                return 0;
        }
    }

    public filterByDuration(
        tracks: SearchTrackItemRdo[],
        duration: SearchDurationFilter
    ): SearchTrackItemRdo[] {
        if (duration === SearchDurationFilter.Any) {
            return tracks;
        }
        return tracks.filter((t) => this.matchesDuration(t.duration, duration));
    }

    public sortByOption(
        tracks: SearchTrackItemRdo[],
        sort: SearchSortOption
    ): SearchTrackItemRdo[] {
        if (sort === SearchSortOption.Relevance) {
            return tracks;
        }
        return [...tracks].sort((a, b) => this.compare(a, b, sort));
    }

    public apply(
        tracks: SearchTrackItemRdo[],
        duration: SearchDurationFilter,
        sort: SearchSortOption
    ): SearchTrackItemRdo[] {
        const filtered = this.filterByDuration(tracks, duration);
        return this.sortByOption(filtered, sort);
    }
}
