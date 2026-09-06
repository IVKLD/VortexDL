export enum SearchDurationFilter {
    Any = 'any',
    Short = 'short',
    Medium = 'medium',
    Long = 'long',
    Epic = 'epic',
}

export enum SearchSortOption {
    Relevance = 'relevance',
    Popularity = 'popularity',
    DurationAsc = 'duration-asc',
    DurationDesc = 'duration-desc',
    TitleAsc = 'title-asc',
}

export interface SortOptionItem<T> {
    label: string;
    shortLabel: string;
    value: T;
    icon: string;
}

export const DURATION_OPTIONS: SortOptionItem<SearchDurationFilter>[] = [
    { label: 'Any length', shortLabel: 'Any length', value: SearchDurationFilter.Any, icon: 'all_inclusive' },
    { label: 'Short (< 2 min)', shortLabel: 'Short (< 2m)', value: SearchDurationFilter.Short, icon: 'timer' },
    { label: 'Medium (2 – 10 min)', shortLabel: 'Medium (2-10m)', value: SearchDurationFilter.Medium, icon: 'schedule' },
    { label: 'Long (10 – 30 min)', shortLabel: 'Long (10-30m)', value: SearchDurationFilter.Long, icon: 'timelapse' },
    { label: 'Mixes / Sets (> 30 min)', shortLabel: 'Mixes (> 30m)', value: SearchDurationFilter.Epic, icon: 'queue_music' },
];

export const SORT_OPTIONS: SortOptionItem<SearchSortOption>[] = [
    { label: 'Relevance', shortLabel: 'Relevance', value: SearchSortOption.Relevance, icon: 'auto_awesome' },
    { label: 'Popularity (Views / Plays)', shortLabel: 'Popularity', value: SearchSortOption.Popularity, icon: 'trending_up' },
    { label: 'Shortest first', shortLabel: 'Shortest', value: SearchSortOption.DurationAsc, icon: 'arrow_upward' },
    { label: 'Longest first', shortLabel: 'Longest', value: SearchSortOption.DurationDesc, icon: 'arrow_downward' },
    { label: 'Title (A-Z)', shortLabel: 'Title (A-Z)', value: SearchSortOption.TitleAsc, icon: 'sort_by_alpha' },
];
