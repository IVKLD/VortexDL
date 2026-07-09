import { DashboardView } from './pages/dashboard-view/dashboard-view';
import { MusicTracksView } from './pages/music-tracks-view/music-tracks-view';
import { Route } from '@angular/router';
import { HeaderConfig, HeaderFeature, HeaderSortOption } from '@shared/components/bricks/header/header.types';
import { SettingsView } from './pages/settings-view/settings-view.component';
import { SearchView } from './pages/search-view/search-view';
import { MusicSortOption, MusicTracksViewState } from './pages/music-tracks-view/music-tracks-view.state';
import { SearchViewState } from './pages/search-view/search-view.state';
import { inject } from '@angular/core';
import { form } from '@angular/forms/signals';

export interface RouteData {
    header?: HeaderConfig;
}

export type CustomRoute = {
    readonly data: RouteData;
} & Route;

export type CustomRoutes = CustomRoute[];

export const routes: CustomRoutes = [
    {
        path: '',
        title: 'Dashboard',
        data: {
            header: {
                title: 'Overview',
                features: [HeaderFeature.Stats],
            },
        },
        component: DashboardView,
    },
    {
        path: 'musics',
        title: 'Musics',
        resolve: {
            headerSearch: () => {
                const state = inject(MusicTracksViewState);
                return {
                    formField: state.searchForm,
                };
            },
            headerSort: () => {
                const state = inject(MusicTracksViewState);
                return {
                    value: state.sortOption,
                    onSortChange: (sort: MusicSortOption) => state.setSortOption(sort),
                };
            },
        },
        data: {
            header: {
                title: 'Library',
                features: [HeaderFeature.Stats, HeaderFeature.Search, HeaderFeature.Sort],
                searchPlaceholder: 'Search in library...',
                sortOptions: [
                    { label: 'Newest First', shortLabel: 'Newest', value: MusicSortOption.DATE_DESC, icon: 'clock_arrow_down' },
                    { label: 'Oldest First', shortLabel: 'Oldest', value: MusicSortOption.DATE_ASC, icon: 'clock_arrow_up' },
                    { label: 'Alphabetical (A-Z)', shortLabel: 'A-Z', value: MusicSortOption.NAME_ASC, icon: 'sort_by_alpha' },
                    { label: 'Alphabetical (Z-A)', shortLabel: 'Z-A', value: MusicSortOption.NAME_DESC, icon: 'sort_by_alpha' },
                ] as HeaderSortOption<MusicSortOption>[],
            },
        },
        component: MusicTracksView,
    },
    {
        path: 'search',
        title: 'Search',
        resolve: {
            headerSearch: () => {
                const state = inject(SearchViewState);
                return {
                    formField: form(state.query),
                    onSubmit: (q: string) => state.search(q),
                    onClear: () => state.clearSearch(),
                };
            },
        },
        data: {
            header: {
                title: 'SoundCloud Search',
                features: [HeaderFeature.Stats, HeaderFeature.Search],
                searchPlaceholder: 'Search SoundCloud...',
            },
        },
        component: SearchView,
    },
    {
        path: 'settings',
        title: 'Settings',
        data: {
            header: {
                title: 'Control Panel',
                features: [HeaderFeature.Stats],
            },
        },
        component: SettingsView,
    },
];
