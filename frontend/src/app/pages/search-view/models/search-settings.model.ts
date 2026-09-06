import { SearchDurationFilter, SearchSortOption } from './search-filter.model';

export enum SearchProvider {
    YouTube = 'youtube',
    SoundCloud = 'soundcloud',
}

export interface SearchSettingsModel {
    provider: SearchProvider;
    duration: SearchDurationFilter;
    sort: SearchSortOption;
}

export const DEFAULT_SEARCH_SETTINGS: SearchSettingsModel = {
    provider: SearchProvider.SoundCloud,
    duration: SearchDurationFilter.Any,
    sort: SearchSortOption.Relevance,
};
